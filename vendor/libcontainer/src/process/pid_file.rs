//! Atomic PID publication without following or truncating an existing destination.
#![forbid(unsafe_code)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug)]
pub struct PublishError {
    pub message: String,
    pub cleanup_failed: bool,
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

pub fn publish(path: &Path, pid: i32) -> Result<(), PublishError> {
    if pid <= 0 {
        return Err(PublishError {
            message: "invalid PID for publication".into(),
            cleanup_failed: false,
        });
    }
    publish_bytes(path, pid.to_string().as_bytes(), 0o666)
}

pub(crate) fn publish_bytes(path: &Path, contents: &[u8], mode: u32) -> Result<(), PublishError> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temporary = tempfile::Builder::new()
        .prefix(".rune-pid-")
        .permissions(std::fs::Permissions::from_mode(mode))
        .tempfile_in(parent)
        .map_err(|error| PublishError {
            message: format!("cannot create PID temporary file: {error}"),
            cleanup_failed: false,
        })?;
    if let Err(error) = temporary
        .write_all(contents)
        .and_then(|()| temporary.as_file().sync_all())
    {
        return Err(cleanup(
            temporary,
            format!("cannot write/synchronize PID file: {error}"),
        ));
    }
    match temporary.persist(path) {
        Ok(_) => Ok(()),
        Err(error) => Err(cleanup(
            error.file,
            format!("cannot publish PID file: {}", error.error),
        )),
    }
}

fn cleanup(temporary: tempfile::NamedTempFile, operation: String) -> PublishError {
    let path = temporary.path().to_path_buf();
    match temporary.close() {
        Ok(()) => PublishError {
            message: operation,
            cleanup_failed: false,
        },
        Err(error) => PublishError {
            message: format!("{operation}; cannot remove PID temporary file {path:?}: {error}"),
            cleanup_failed: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn replaces_destination_without_truncating_its_old_inode_or_symlink_target() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("pid");
        let old = root.path().join("old");
        fs::write(&destination, "original").unwrap();
        fs::hard_link(&destination, &old).unwrap();
        publish(&destination, 123).unwrap();
        assert_eq!(fs::read_to_string(&destination).unwrap(), "123");
        assert_eq!(fs::read_to_string(&old).unwrap(), "original");
        fs::remove_file(&destination).unwrap();
        std::os::unix::fs::symlink(&old, &destination).unwrap();
        publish(&destination, 456).unwrap();
        assert!(
            !fs::symlink_metadata(&destination)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_to_string(&old).unwrap(), "original");
    }
    #[test]
    fn failure_preserves_destination_and_cleans_temporary_files() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("directory");
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("retained"), "owned").unwrap();
        let failure = publish(&directory, 123).unwrap_err();
        assert!(!failure.cleanup_failed);
        assert_eq!(
            fs::read_to_string(directory.join("retained")).unwrap(),
            "owned"
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
        assert!(publish(&root.path().join("missing/pid"), 123).is_err());
        assert!(publish(&root.path().join("pid"), 0).is_err());
    }
    #[test]
    fn concurrent_publishers_leave_one_complete_record_and_no_temporaries() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("pid");
        publish(&destination, 111111).unwrap();
        std::thread::scope(|scope| {
            for pid in [111111, 222222, 333333, 444444] {
                let destination = &destination;
                scope.spawn(move || {
                    for _ in 0..25 {
                        publish(destination, pid).unwrap();
                    }
                });
            }
            let destination = &destination;
            scope.spawn(move || {
                for _ in 0..1000 {
                    let record = fs::read_to_string(destination).unwrap();
                    assert!(["111111", "222222", "333333", "444444"].contains(&record.as_str()));
                }
            });
        });
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }
}
