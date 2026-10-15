use std::fs;
use std::process::Command;

#[test]
fn cli_errors_and_backend_events_use_requested_format_and_append() {
    for format in ["text", "json"] {
        for debug in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let log = temp.path().join("diagnostics");
            fs::write(&log, "previous record\n").unwrap();
            let mut command = Command::new(env!("CARGO_BIN_EXE_rune"));
            command.args(["--log-format", format, "--log"]).arg(&log);
            if debug {
                command.arg("--debug");
            }
            let output = command
                .arg("--root")
                .arg(temp.path().join("state"))
                .args(["state", "missing"])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8(output.stderr).unwrap().contains("rune:"));
            let text = fs::read_to_string(&log).unwrap();
            assert!(text.starts_with("previous record\n"));
            let lines: Vec<_> = text.lines().skip(1).collect();
            // State::load emits a backend error, followed by the final chain.
            assert!(lines.len() >= 2, "missing backend/final diagnostic");
            for line in lines {
                if format == "json" {
                    let record: serde_json::Value = serde_json::from_str(line).unwrap();
                    assert_eq!(record["level"], "error");
                    assert!(record["msg"].is_string());
                    chrono::DateTime::parse_from_rfc3339(record["time"].as_str().unwrap()).unwrap();
                } else {
                    assert!(line.starts_with("time="));
                    assert!(line.contains("level=error"));
                }
            }
        }
    }
}

#[test]
fn stderr_json_uses_same_schema_without_a_file() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rune"))
        .args(["--log-format", "json", "--root"])
        .arg(temp.path())
        .args(["state", "missing"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    for line in String::from_utf8(output.stderr).unwrap().lines() {
        let record: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(record["level"], "error");
        chrono::DateTime::parse_from_rfc3339(record["time"].as_str().unwrap()).unwrap();
    }
}

#[test]
fn invalid_log_destination_fails_before_runtime_state_creation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("state");
    let output = Command::new(env!("CARGO_BIN_EXE_rune"))
        .arg("--log")
        .arg(temp.path())
        .arg("--root")
        .arg(&root)
        .args(["delete", "--force", "missing"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot initialize logging")
    );
    assert!(!root.exists());
}

#[test]
fn log_write_failure_is_visible_alongside_the_command_error() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rune"))
        .args(["--log", "/dev/full", "--root"])
        .arg(temp.path())
        .args(["state", "missing"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("rune:"));
    assert!(stderr.contains("cannot write requested log"));
}
