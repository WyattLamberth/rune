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

#[test]
fn numa_policy_errors_do_not_create_container_ownership_records() {
    let policies = [
        serde_json::json!({"mode":"MPOL_BIND", "nodes":"0-4294967295"}),
        serde_json::json!({"mode":"MPOL_BIND", "nodes":"4294967295"}),
        serde_json::json!({"mode":"MPOL_BIND", "nodes":"0,".repeat(2_097_153)}),
        serde_json::json!({"mode":"MPOL_BIND", "nodes":"2-1"}),
        serde_json::json!({"mode":"MPOL_BIND"}),
        serde_json::json!({"mode":"MPOL_LOCAL", "nodes":"1"}),
        serde_json::json!({"mode":"MPOL_BIND", "nodes":"0", "flags":["MPOL_F_STATIC_NODES", "MPOL_F_RELATIVE_NODES"]}),
        serde_json::json!({"mode":"MPOL_LOCAL", "flags":["MPOL_F_NUMA_BALANCING"]}),
    ];
    for policy in policies {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("state");
        let bundle = temp.path().join("bundle");
        fs::create_dir_all(bundle.join("rootfs")).unwrap();
        let mut spec = serde_json::to_value(oci_spec::runtime::Spec::default()).unwrap();
        spec["ociVersion"] = serde_json::json!("1.3.0");
        spec["process"]["terminal"] = serde_json::json!(false);
        spec["process"]["args"] = serde_json::json!(["/bin/true"]);
        spec["linux"]["memoryPolicy"] = policy;
        fs::write(
            bundle.join("config.json"),
            serde_json::to_vec(&spec).unwrap(),
        )
        .unwrap();
        let id = format!(
            "bounds-{}",
            temp.path().file_name().unwrap().to_str().unwrap()
        );
        let result = Command::new(env!("CARGO_BIN_EXE_rune"))
            .arg("--root")
            .arg(&root)
            .arg("create")
            .arg("--bundle")
            .arg(&bundle)
            .arg(&id)
            .output()
            .unwrap();
        assert!(!result.status.success());
        let error = String::from_utf8(result.stderr).unwrap();
        assert!(
            error.contains("node")
                || error.contains("flag")
                || error.contains("mode")
                || error.contains("4 MiB")
        );
        assert!(result.stdout.is_empty());
        assert!(
            !root.join(&id).exists(),
            "policy rejection created ownership records"
        );
    }
}

fn deadline_output(command: &mut Command) -> std::process::Output {
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("owned CLI command exceeded deadline");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn forced_absence_is_idempotent_but_partial_or_nonregular_ownership_is_not() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    for _ in 0..2 {
        let output = deadline_output(
            Command::new(env!("CARGO_BIN_EXE_rune"))
                .arg("--root")
                .arg(root.path())
                .args(["delete", "--force", "absent"]),
        );
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!root.path().join("absent").exists());
    }
    let output = deadline_output(
        Command::new(env!("CARGO_BIN_EXE_rune"))
            .arg("--root")
            .arg(root.path())
            .args(["delete", "absent"]),
    );
    assert!(!output.status.success());
    for id in ["../outside", ".rune-locks", "a/b"] {
        let output = deadline_output(
            Command::new(env!("CARGO_BIN_EXE_rune"))
                .arg("--root")
                .arg(root.path())
                .args(["delete", "--force", id]),
        );
        assert!(!output.status.success());
    }
    let partial = root.path().join("partial");
    fs::create_dir(&partial).unwrap();
    fs::write(partial.join("ownership"), b"retain").unwrap();
    let file = root.path().join("file");
    fs::write(&file, b"retain").unwrap();
    symlink(&partial, root.path().join("link")).unwrap();
    for id in ["partial", "file", "link"] {
        let output = deadline_output(
            Command::new(env!("CARGO_BIN_EXE_rune"))
                .arg("--root")
                .arg(root.path())
                .args(["delete", "--force", id]),
        );
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    assert_eq!(fs::read(partial.join("ownership")).unwrap(), b"retain");
    assert_eq!(fs::read(file).unwrap(), b"retain");
    assert!(root.path().join("link").is_symlink());
}
