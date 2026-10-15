use std::collections::HashMap;
use std::io::{ErrorKind, Write};
use std::os::fd::AsRawFd;
use std::os::unix::prelude::CommandExt;
use std::path::Path;
use std::{process, thread, time};

use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::unistd::Pid;
use oci_spec::runtime::{Hook, State as OciState};

use crate::container::{State, StateConversionError};
use crate::utils;

#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("failed to execute hook command")]
    CommandExecute(#[source] std::io::Error),
    #[error("failed to encode container state")]
    EncodeContainerState(#[source] serde_json::Error),
    #[error("hook command exited with non-zero exit code: {0}")]
    NonZeroExitCode(i32),
    #[error("hook command was killed by a signal")]
    Killed,
    #[error("hook timeout must be nonnegative and representable")]
    InvalidTimeout,
    #[error("hook failure {cause}; child cleanup failed (kill: {kill:?}, wait: {wait:?})")]
    Cleanup {
        cause: Box<HookError>,
        kill: Option<std::io::Error>,
        wait: Option<std::io::Error>,
    },
    #[error("failed to execute hook command due to a timeout")]
    Timeout,
    #[error("container state is required to run hook")]
    MissingContainerState,
    #[error("failed to write container state to stdin")]
    WriteContainerState(#[source] std::io::Error),
    #[error("failed to convert state to OCI format")]
    StateConversion(#[from] StateConversionError),
}

type Result<T> = std::result::Result<T, HookError>;

pub fn run_hooks(
    hooks: Option<&Vec<Hook>>,
    state: Option<&State>,
    // TODO: Remove the following parameters. To comply with the OCI State, hooks should only depend on structures defined in oci-spec-rs. Cleaning these up ensures proper functional isolation.
    cwd: Option<&Path>,
    pid: Option<Pid>,
    default_env: Option<&HashMap<String, String>>,
) -> Result<()> {
    let base_state = state.ok_or(HookError::MissingContainerState)?;

    // High-level container runtimes use OCI state to pass the container state to the hooks.
    // So we need to convert the container state to OCI state.
    // Ref: https://github.com/containerd/containerd/blob/v2.2.1/cmd/containerd/command/oci-hook.go#L82
    let mut oci_state = OciState::try_from(base_state)?;

    // The `pid` parameter allows overriding the PID in the state. This is needed because
    // high-level container runtimes like containerd set the PID separately for certain hooks.
    // Ref: https://github.com/containerd/containerd/blob/main/cmd/containerd/command/oci-hook.go#L90
    if let Some(override_pid) = pid {
        oci_state.set_pid(Some(override_pid.as_raw()));
    }

    if let Some(hooks) = hooks {
        for hook in hooks {
            let mut hook_command = process::Command::new(hook.path());

            if let Some(cwd) = cwd {
                hook_command.current_dir(cwd);
            }

            // Based on OCI spec, the first argument of the args vector is the
            // arg0, which can be different from the path.  For example, path
            // may be "/usr/bin/true" and arg0 is set to "true". However, rust
            // command differentiates arg0 from args, where rust command arg
            // doesn't include arg0. So we have to make the split arg0 from the
            // rest of args.
            if let Some((arg0, args)) = hook.args().as_ref().and_then(|a| a.split_first()) {
                tracing::debug!("run_hooks arg0: {:?}, args: {:?}", arg0, args);
                hook_command.arg0(arg0).args(args)
            } else {
                hook_command.arg0(hook.path().display().to_string())
            };

            let envs: HashMap<String, String> = if let Some(env) = hook.env() {
                utils::parse_env(env)
            } else if let Some(default) = default_env {
                default.clone()
            } else {
                HashMap::new()
            };
            tracing::debug!("run_hooks envs: {:?}", envs);

            // Encode before spawning so encoding failure cannot abandon a child.
            let encoded_state =
                serde_json::to_vec(&oci_state).map_err(HookError::EncodeContainerState)?;
            let deadline = match hook.timeout() {
                Some(seconds) => {
                    let seconds = u64::try_from(seconds).map_err(|_| HookError::InvalidTimeout)?;
                    Some(
                        time::Instant::now()
                            .checked_add(time::Duration::from_secs(seconds))
                            .ok_or(HookError::InvalidTimeout)?,
                    )
                }
                None => None,
            };
            let mut hook_process = hook_command
                .env_clear()
                .envs(envs)
                .stdin(process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(process::Stdio::inherit())
                .spawn()
                .map_err(HookError::CommandExecute)?;
            let result = deliver_and_wait(&mut hook_process, &encoded_state, deadline);
            let exit_status = match result {
                Ok(status) => status,
                Err(cause) => return Err(terminate_and_reap(&mut hook_process, cause)),
            };
            match exit_status.code() {
                Some(0) => {}
                Some(exit_code) => return Err(HookError::NonZeroExitCode(exit_code)),
                None => return Err(HookError::Killed),
            }
        }
    }

    Ok(())
}

// One owner retains Child until it has been reaped. No detached waiter or bare
// PID signaling can race a reap and accidentally target a reused process ID.
fn terminate_and_reap(child: &mut process::Child, cause: HookError) -> HookError {
    let kill = child.kill().err();
    let wait = child.wait().err();
    if kill.is_none() && wait.is_none() {
        cause
    } else {
        HookError::Cleanup {
            cause: Box::new(cause),
            kill,
            wait,
        }
    }
}

fn check_deadline(deadline: Option<time::Instant>) -> Result<()> {
    if deadline.is_some_and(|value| time::Instant::now() >= value) {
        Err(HookError::Timeout)
    } else {
        Ok(())
    }
}

fn pause(deadline: Option<time::Instant>) {
    let interval = time::Duration::from_millis(5);
    thread::sleep(deadline.map_or(interval, |value| {
        interval.min(value.saturating_duration_since(time::Instant::now()))
    }));
}

fn deliver_and_wait(
    child: &mut process::Child,
    state: &[u8],
    deadline: Option<time::Instant>,
) -> Result<process::ExitStatus> {
    if let Some(mut stdin) = child.stdin.take() {
        // Keep the existing pipe semantics, but never block delivery outside the
        // deadline. nix exposes safe fcntl; this adds no unsafe code.
        let flags = fcntl(stdin.as_raw_fd(), FcntlArg::F_GETFL)
            .map_err(|error| HookError::WriteContainerState(error.into()))?;
        fcntl(
            stdin.as_raw_fd(),
            FcntlArg::F_SETFL(OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK),
        )
        .map_err(|error| HookError::WriteContainerState(error.into()))?;
        let mut remaining = state;
        while !remaining.is_empty() {
            check_deadline(deadline)?;
            match stdin.write(remaining) {
                Ok(0) => {
                    return Err(HookError::WriteContainerState(std::io::Error::from(
                        ErrorKind::WriteZero,
                    )))
                }
                Ok(written) => remaining = &remaining[written..],
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == ErrorKind::WouldBlock => pause(deadline),
                // A hook may legitimately exit without reading its state. Its
                // status below remains authoritative, as in the prior adapter.
                Err(error) if error.kind() == ErrorKind::BrokenPipe => break,
                Err(error) => return Err(HookError::WriteContainerState(error)),
            }
        }
        // Drop the writer before waiting: hooks consuming all stdin require EOF.
    }
    loop {
        if let Some(status) = child.try_wait().map_err(HookError::CommandExecute)? {
            return Ok(status);
        }
        check_deadline(deadline)?;
        pause(deadline);
    }
}

#[cfg(test)]
mod test {
    use std::{env, fs};

    use anyhow::{bail, Context, Result};
    use oci_spec::runtime::HookBuilder;
    use serial_test::serial;

    use super::*;
    use crate::container::Container;

    fn is_command_in_path(program: &str) -> bool {
        if let Ok(path) = env::var("PATH") {
            for p in path.split(':') {
                let p_str = format!("{p}/{program}");
                if fs::metadata(p_str).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    // Note: the run_hook will require the use of pipe to write the container
    // state into stdin of the hook command. When cargo test runs these tests in
    // parallel with other tests, the pipe becomes flaky and often we will get
    // broken pipe or bad file descriptors. There is not much we can do and we
    // decide not to retry in the test. The most sensible way to test this is
    // ask cargo test to run these tests in serial.

    #[test]
    #[serial]
    fn test_run_hook() -> Result<()> {
        {
            let default_container: Container = Default::default();
            run_hooks(None, Some(&default_container.state), None, None, None)
                .context("Failed simple test")?;
        }

        {
            assert!(is_command_in_path("true"), "The true was not found.");
            let default_container: Container = Default::default();

            let hook = HookBuilder::default().path("true").build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                Some(&default_container.state),
                None,
                None,
                None,
            )
            .context("Failed true")?;
        }

        {
            assert!(
                is_command_in_path("printenv"),
                "The printenv was not found."
            );
            // Use `printenv` to make sure the environment is set correctly.
            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("bash")
                .args(vec![
                    String::from("bash"),
                    String::from("-c"),
                    String::from("printenv key > /dev/null"),
                ])
                .env(vec![String::from("key=value")])
                .build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                Some(&default_container.state),
                None,
                None,
                None,
            )
            .context("Failed printenv test")?;
        }

        {
            assert!(is_command_in_path("pwd"), "The pwd was not found.");

            let tmp = tempfile::tempdir()?;

            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("bash")
                .args(vec![
                    String::from("bash"),
                    String::from("-c"),
                    format!("test $(pwd) = {:?}", tmp.path()),
                ])
                .build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                Some(&default_container.state),
                Some(tmp.path()),
                None,
                None,
            )
            .context("Failed pwd test")?;
        }

        {
            let default_container: Container = Default::default();
            let expected_pid = Pid::from_raw(1000);

            let hook = HookBuilder::default()
                .path("bash")
                .args(vec![
                    String::from("bash"),
                    String::from("-c"),
                    format!("cat | grep '\"pid\":{}'", expected_pid),
                ])
                .build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                Some(&default_container.state),
                None,
                Some(expected_pid),
                None,
            )
            .context("Failed pid test")?;
        }

        Ok(())
    }

    #[test]
    #[serial]
    // This will test executing hook with a timeout. Since the timeout is set in
    // secs, minimally, the test will run for 1 second to trigger the timeout.
    fn test_run_hook_timeout() -> Result<()> {
        let default_container: Container = Default::default();
        // We use `tail -f /dev/null` here to simulate a hook command that hangs.
        let hook = HookBuilder::default()
            .path("tail")
            .args(vec![
                String::from("tail"),
                String::from("-f"),
                String::from("/dev/null"),
            ])
            .timeout(1)
            .build()?;
        let hooks = Some(vec![hook]);
        match run_hooks(
            hooks.as_ref(),
            Some(&default_container.state),
            None,
            None,
            None,
        ) {
            Ok(_) => {
                bail!(
                    "The test expects the hook to error out with timeout. Should not execute cleanly"
                );
            }
            Err(HookError::Timeout) => {}
            Err(err) => {
                bail!(
                    "The test expects the hook to error out with timeout. Got error: {}",
                    err
                );
            }
        };

        Ok(())
    }

    #[test]
    #[serial]
    fn test_run_hook_default_env() -> Result<()> {
        // Test: hook without explicit env uses default_env
        {
            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("sh")
                .args(vec![
                    String::from("sh"),
                    String::from("-c"),
                    String::from("test \"$TEST_ENV\" = 'default_value'"),
                ])
                .build()?;
            let hooks = Some(vec![hook]);
            let mut default_env = HashMap::new();
            default_env.insert("TEST_ENV".to_string(), "default_value".to_string());
            run_hooks(
                hooks.as_ref(),
                Some(&default_container.state),
                None,
                None,
                Some(&default_env),
            )
            .context("Failed: hook without explicit env should use default_env")?;
        }

        // Test: hook with explicit env takes priority over default_env
        {
            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("sh")
                .args(vec![
                    String::from("sh"),
                    String::from("-c"),
                    String::from("test \"$TEST_ENV\" = 'explicit_value'"),
                ])
                .env(vec![String::from("TEST_ENV=explicit_value")])
                .build()?;
            let hooks = Some(vec![hook]);
            let mut default_env = HashMap::new();
            default_env.insert("TEST_ENV".to_string(), "default_value".to_string());
            run_hooks(
                hooks.as_ref(),
                Some(&default_container.state),
                None,
                None,
                Some(&default_env),
            )
            .context("Failed: hook with explicit env should ignore default_env")?;
        }

        // Test: hook without explicit env and no default_env gets empty environment
        {
            let default_container: Container = Default::default();
            let hook = HookBuilder::default()
                .path("sh")
                .args(vec![
                    String::from("sh"),
                    String::from("-c"),
                    // Verify that the environment is empty (no TEST_ENV, etc.)
                    String::from("test -z \"$TEST_ENV\""),
                ])
                .build()?;
            let hooks = Some(vec![hook]);
            run_hooks(
                hooks.as_ref(),
                Some(&default_container.state),
                None,
                None,
                None,
            )
            .context("Failed: hook without env and without default_env should have empty env")?;
        }

        Ok(())
    }
}
