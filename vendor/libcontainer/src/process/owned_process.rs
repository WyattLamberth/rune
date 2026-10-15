//! Roll back only a process proven to be an unreaped child of this caller.
#![forbid(unsafe_code)]

use nix::errno::Errno;
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::Pid;
use std::time::{Duration, Instant};

pub fn terminate_and_reap(pid: Pid) -> Result<(), String> {
    // A successful child wait check keeps the PID reserved until our reap,
    // including if the process exits before pidfd_open. Never signal ECHILD.
    match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::Exited(_, _) | WaitStatus::Signaled(_, _, _)) => return Ok(()),
        Ok(WaitStatus::StillAlive | WaitStatus::Stopped(_, _) | WaitStatus::Continued(_)) => {}
        Ok(status) => return Err(format!("unexpected owned-process status: {status:?}")),
        Err(error) => return Err(format!("cannot establish child ownership: {error}")),
    }
    let process = rustix::process::Pid::from_raw(pid.as_raw()).ok_or("invalid owned PID")?;
    let fd = rustix::process::pidfd_open(process, rustix::process::PidfdFlags::empty())
        .map_err(|error| format!("cannot acquire owned process handle: {error}"))?;
    let signal_error = match rustix::process::pidfd_send_signal(&fd, rustix::process::Signal::KILL)
    {
        Ok(()) | Err(rustix::io::Errno::SRCH) => None,
        Err(error) => Some(format!("cannot terminate owned process: {error}")),
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(_, _) | WaitStatus::Signaled(_, _, _)) => {
                return match signal_error {
                    Some(error) => Err(error),
                    None => Ok(()),
                };
            }
            Ok(WaitStatus::StillAlive | WaitStatus::Stopped(_, _) | WaitStatus::Continued(_))
            | Err(Errno::EINTR) => {}
            result => {
                return Err(format!(
                    "{}cannot confirm owned-process reap: {result:?}",
                    signal_error
                        .map(|error| format!("{error}; "))
                        .unwrap_or_default()
                ));
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "{}owned-process reap deadline exceeded",
                signal_error
                    .map(|error| format!("{error}; "))
                    .unwrap_or_default()
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    #[test]
    fn live_owned_child_is_terminated_and_reaped() {
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let pid = Pid::from_raw(child.id() as i32);
        let result = terminate_and_reap(pid);
        let after = waitpid(pid, Some(WaitPidFlag::WNOHANG));
        if result.is_err() || after == Ok(WaitStatus::StillAlive) {
            let _ = child.kill();
        }
        // The helper already reaps on success; always wait as a fallback on failure.
        let _ = child.wait();
        result.unwrap();
        assert_eq!(after, Err(Errno::ECHILD));
    }
    #[test]
    fn unowned_process_is_never_signalled() {
        assert!(
            terminate_and_reap(nix::unistd::getpid())
                .unwrap_err()
                .contains("ownership")
        );
    }
}
