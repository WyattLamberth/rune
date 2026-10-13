//! Finalize rollback only after resource teardown succeeds.
#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;

pub(super) fn finish_cleanup(root: Option<&Path>, errors: Vec<String>) -> Result<(), String> {
    if !errors.is_empty() {
        return Err(format!("failed to cleanup container: {}", errors.join(";")));
    }
    if let Some(root) = root {
        match fs::remove_dir_all(root) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "failed to remove container ownership records: {error}"
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_failure_retains_records_and_successful_retry_removes_them() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("owned");
        fs::create_dir(&root).unwrap();
        let records = root.join("state.json");
        fs::write(&records, b"ownership evidence").unwrap();
        let error = finish_cleanup(
            Some(&root),
            vec!["cgroup failure".into(), "RDT failure".into()],
        )
        .unwrap_err();
        assert!(error.contains("cgroup failure") && error.contains("RDT failure"));
        assert_eq!(fs::read(&records).unwrap(), b"ownership evidence");
        finish_cleanup(Some(&root), Vec::new()).unwrap();
        assert!(!root.exists());
        finish_cleanup(Some(&root), Vec::new()).unwrap();
    }

    #[test]
    fn record_removal_failure_is_returned_and_retains_the_input() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("non-directory");
        fs::write(&file, b"retained").unwrap();
        assert!(finish_cleanup(Some(&file), Vec::new()).is_err());
        assert_eq!(fs::read(&file).unwrap(), b"retained");
    }

    #[test]
    fn resource_errors_survive_without_a_state_directory() {
        assert!(
            finish_cleanup(None, vec!["resource failure".into()])
                .unwrap_err()
                .contains("resource failure")
        );
        finish_cleanup(None, Vec::new()).unwrap();
    }
}
