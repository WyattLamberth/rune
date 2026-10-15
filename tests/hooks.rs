//! Exercise the production dependency hook adapter with real, bounded children.
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use libcontainer::container::State;
use libcontainer::hooks::{HookError, run_hooks};
use oci_spec::runtime::HookBuilder;

#[test]
#[ignore = "bounded subprocess worker invoked by the parent test"]
fn hook_worker() {
    let case = std::env::var("RUNE_HOOK_CASE").unwrap();
    let directory = std::env::var("RUNE_HOOK_DIRECTORY").unwrap();
    let script = if case == "backpressure" || case == "wait-timeout" {
        "import pathlib,sys,time,os;pathlib.Path(sys.argv[1]).write_text(str(os.getpid()));time.sleep(4)"
    } else if case == "nonzero" {
        "import pathlib,sys,os;pathlib.Path(sys.argv[1]).write_text(str(os.getpid()));sys.exit(7)"
    } else if case == "early-success" {
        "import pathlib,sys,os;pathlib.Path(sys.argv[1]).write_text(str(os.getpid()));sys.exit(0)"
    } else {
        "import pathlib,sys,os;pathlib.Path(sys.argv[1]).write_text(str(os.getpid()));sys.stdin.read()"
    };
    let hook = HookBuilder::default()
        .path("/usr/bin/python3")
        .args(vec![
            "python3".into(),
            "-c".into(),
            script.into(),
            format!("{directory}/pid"),
        ])
        .timeout(if case == "negative" { -1 } else { 1 })
        .build()
        .unwrap();
    let state = State {
        annotations: Some(HashMap::from([(
            "bounded-test".into(),
            "x".repeat(if case == "wait-timeout" {
                1
            } else {
                256 * 1024
            }),
        )])),
        ..State::default()
    };
    let started = Instant::now();
    let result = run_hooks(Some(&vec![hook]), Some(&state), None, None, None);
    if case == "backpressure" || case == "wait-timeout" {
        assert!(
            matches!(result, Err(HookError::Timeout)),
            "deadline did not cover state delivery"
        );
        assert!(started.elapsed() < Duration::from_secs(3));
    } else if case == "negative" {
        assert!(matches!(result, Err(HookError::InvalidTimeout)));
        assert!(!std::path::Path::new(&format!("{directory}/pid")).exists());
        return;
    } else if case == "nonzero" {
        assert!(matches!(result, Err(HookError::NonZeroExitCode(7))));
    } else {
        result.expect("stdin must close after delivering state so the hook sees EOF");
    }
    let pid = std::fs::read_to_string(format!("{directory}/pid")).unwrap();
    assert!(
        !std::path::Path::new(&format!("/proc/{pid}")).exists(),
        "owned child must be reaped"
    );
}

#[test]
fn deadlines_cover_backpressure_and_stdin_is_closed() {
    for case in [
        "backpressure",
        "eof",
        "wait-timeout",
        "nonzero",
        "early-success",
        "negative",
    ] {
        let directory = tempfile::tempdir().unwrap();
        let mut worker = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "hook_worker", "--ignored"])
            .env("RUNE_HOOK_CASE", case)
            .env("RUNE_HOOK_DIRECTORY", directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let started = Instant::now();
        let status = loop {
            if let Some(status) = worker.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > Duration::from_secs(7) {
                worker.kill().unwrap();
                worker.wait().unwrap();
                panic!("bounded hook worker exceeded its deadline: {case}");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(
            status.success(),
            "production hook regression failed: {case}"
        );
    }
}
