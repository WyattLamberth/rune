use std::fs;
use std::os::unix::fs::MetadataExt;
use std::process::{Child, Command};

struct OwnedProcess(Child);
impl Drop for OwnedProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn fixture(root: &std::path::Path, pid: u32, starttime: u64) {
    let state = root.join("test");
    fs::create_dir(&state).unwrap();
    fs::write(
        state.join("state.json"),
        serde_json::to_vec(&serde_json::json!({
            "ociVersion":"1.2.1", "id":"test", "status":"running", "pid":pid,
            "runePidStarttime":starttime,"bundle":root,"useSystemd":false
        }))
        .unwrap(),
    )
    .unwrap();
}
fn starttime(pid: u32) -> u64 {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
    stat.rsplit_once(')')
        .unwrap()
        .1
        .split_whitespace()
        .nth(19)
        .unwrap()
        .parse()
        .unwrap()
}
#[test]
fn stale_process_identity_is_observed_as_stopped_and_cannot_be_signalled() {
    let root = tempfile::tempdir().unwrap();
    let mut child = OwnedProcess(Command::new("sleep").arg("30").spawn().unwrap());
    fixture(root.path(), child.0.id(), starttime(child.0.id()) + 1);
    let state = Command::new(env!("CARGO_BIN_EXE_rune"))
        .arg("--root")
        .arg(root.path())
        .args(["state", "test"])
        .output()
        .unwrap();
    assert!(state.status.success(), "{:?}", state.stderr);
    let state: serde_json::Value = serde_json::from_slice(&state.stdout).unwrap();
    assert_eq!(state["status"], "stopped");
    assert!(state["pid"].is_null());
    let kill = Command::new(env!("CARGO_BIN_EXE_rune"))
        .arg("--root")
        .arg(root.path())
        .args(["kill", "test", "TERM"])
        .output()
        .unwrap();
    assert!(!kill.status.success());
    assert!(child.0.try_wait().unwrap().is_none());
}
#[test]
fn stable_process_handle_signals_owned_process_and_commits_state_atomically() {
    let root = tempfile::tempdir().unwrap();
    let mut child = OwnedProcess(Command::new("sleep").arg("30").spawn().unwrap());
    fixture(root.path(), child.0.id(), starttime(child.0.id()));
    let path = root.path().join("test/state.json");
    let old_inode = fs::metadata(&path).unwrap().ino();
    let kill = Command::new(env!("CARGO_BIN_EXE_rune"))
        .arg("--root")
        .arg(root.path())
        .args(["kill", "test", "TERM"])
        .output()
        .unwrap();
    assert!(kill.status.success(), "{:?}", kill.stderr);
    assert!(!child.0.wait().unwrap().success());
    let state: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(state["status"], "stopped");
    assert_ne!(old_inode, fs::metadata(&path).unwrap().ino());
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o600);
}
#[test]
fn unsupported_flags_and_missing_containers_fail_without_success_json() {
    let root = tempfile::tempdir().unwrap();
    for args in [
        vec!["--invented-flag", "state", "test"],
        vec!["state", "absent"],
        vec!["state", "../test"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_rune"))
            .arg("--root")
            .arg(root.path())
            .args(args)
            .output()
            .unwrap();
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn delete_retains_state_and_owned_process_when_cleanup_metadata_is_unusable() {
    for content in [
        None,
        Some(b"not-json".as_slice()),
        Some(b"{\"cgroup_path\":\"x\",\"unknown\":true}".as_slice()),
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut child = OwnedProcess(Command::new("sleep").arg("30").spawn().unwrap());
        fixture(root.path(), child.0.id(), starttime(child.0.id()) + 1);
        let state_path = root.path().join("test/state.json");
        let original_state = fs::read(&state_path).unwrap();
        if let Some(content) = content {
            fs::write(root.path().join("test/youki_config.json"), content).unwrap();
        }
        let deleted = Command::new(env!("CARGO_BIN_EXE_rune"))
            .arg("--root")
            .arg(root.path())
            .args(["delete", "--force", "test"])
            .output()
            .unwrap();
        assert!(
            !deleted.status.success(),
            "missing cleanup ownership must not be successful deletion"
        );
        assert_eq!(fs::read(&state_path).unwrap(), original_state);
        assert!(child.0.try_wait().unwrap().is_none());
    }
}

#[test]
fn cleanup_metadata_rejects_oversized_files_symlinks_and_nonregular_inputs() {
    use std::os::unix::fs::symlink;
    for kind in ["oversized", "symlink", "directory", "fifo"] {
        let root = tempfile::tempdir().unwrap();
        let mut child = OwnedProcess(Command::new("sleep").arg("30").spawn().unwrap());
        fixture(root.path(), child.0.id(), starttime(child.0.id()) + 1);
        let metadata = root.path().join("test/youki_config.json");
        match kind {
            "oversized" => fs::write(&metadata, vec![b' '; 4 * 1024 * 1024 + 1]).unwrap(),
            "symlink" => {
                let target = root.path().join("target.json");
                fs::write(&target, b"{}").unwrap();
                symlink(target, &metadata).unwrap();
            }
            "directory" => fs::create_dir(&metadata).unwrap(),
            "fifo" => nix::unistd::mkfifo(
                &metadata,
                nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
            )
            .unwrap(),
            _ => unreachable!(),
        }
        let deleted = Command::new(env!("CARGO_BIN_EXE_rune"))
            .arg("--root")
            .arg(root.path())
            .args(["delete", "--force", "test"])
            .output()
            .unwrap();
        assert!(!deleted.status.success());
        assert!(root.path().join("test/state.json").is_file());
        assert!(child.0.try_wait().unwrap().is_none());
    }
}
