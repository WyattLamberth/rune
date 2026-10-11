use libcgroups::common::{CgroupManager, get_cgroup_setup};
use nix::sys::signal::{self};

use super::{Container, ContainerStatus};
use crate::error::LibcontainerError;
use crate::signal::Signal;

impl Container {
    /// Sends the specified signal to the container init process
    ///
    /// # Example
    ///
    /// ```no_run
    /// use libcontainer::container::builder::ContainerBuilder;
    /// use libcontainer::syscall::syscall::SyscallType;
    /// use nix::sys::signal::Signal;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut container = ContainerBuilder::new(
    ///     "74f1a4cb3801".to_owned(),
    ///     SyscallType::default(),
    /// )
    /// .as_init("/var/run/docker/bundle")
    /// .build()?;
    ///
    /// container.kill(Signal::SIGKILL, false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn kill<S: Into<Signal>>(&mut self, signal: S, all: bool) -> Result<(), LibcontainerError> {
        self.refresh_status()?;
        match self.can_kill() {
            true => {
                self.do_kill(signal, all)?;
            }
            false if all && self.status() == ContainerStatus::Stopped => {
                self.do_kill(signal, all)?;
            }
            false => {
                tracing::error!(id = ?self.id(), status = ?self.status(), "cannot kill container due to incorrect state");
                return Err(LibcontainerError::IncorrectStatus(self.status()));
            }
        }
        self.set_status(ContainerStatus::Stopped).save()?;
        Ok(())
    }

    pub(crate) fn do_kill<S: Into<Signal>>(
        &self,
        signal: S,
        all: bool,
    ) -> Result<(), LibcontainerError> {
        if all {
            self.kill_all_processes(signal)
        } else {
            self.kill_one_process(signal)
        }
    }

    fn kill_one_process<S: Into<Signal>>(&self, signal: S) -> Result<(), LibcontainerError> {
        let signal = signal.into().into_raw();
        let pid = self.pid().ok_or(LibcontainerError::Other(
            "container process pid not found in state".into(),
        ))?;

        tracing::debug!("kill signal {} to {}", signal, pid);

        let rustix_pid = rustix::process::Pid::from_raw(pid.as_raw()).ok_or_else(|| LibcontainerError::Other("invalid process ID".into()))?;
        let fd = match rustix::process::pidfd_open(rustix_pid, rustix::process::PidfdFlags::empty()) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::SRCH) => return Ok(()),
            Err(err) => return Err(LibcontainerError::OtherIO(err.into())),
        };
        let observed = procfs::process::Process::new(pid.as_raw()).ok().and_then(|p| p.stat().ok()).map(|s| s.starttime);
        if observed.is_none() { return Ok(()); }
        if observed != self.state.rune_pid_starttime {
            return Err(LibcontainerError::Other("process identity changed; refusing to signal".into()));
        }
        let rustix_signal = rustix::process::Signal::from_named_raw(signal as i32).ok_or_else(|| LibcontainerError::Other("unsupported signal".into()))?;
        match rustix::process::pidfd_send_signal(&fd, rustix_signal).map_err(|err| nix::errno::Errno::from_raw(err.raw_os_error())) {
            Ok(_) => {}
            Err(nix::errno::Errno::ESRCH) => {
                // the process does not exist, which is what we want
            }
            Err(err) => {
                tracing::error!(id = ?self.id(), err = ?err, ?pid, ?signal, "failed to kill process");
                return Err(LibcontainerError::OtherSyscall(err));
            }
        }

        // For cgroup V1, a frozon process cannot respond to signals,
        // so we need to thaw it. Only thaw the cgroup for SIGKILL.
        if self.status() == ContainerStatus::Paused && signal == signal::Signal::SIGKILL {
            match get_cgroup_setup()? {
                libcgroups::common::CgroupSetup::Legacy
                | libcgroups::common::CgroupSetup::Hybrid => {
                    let cmanager = libcgroups::common::create_cgroup_manager(
                        libcgroups::common::CgroupConfig {
                            cgroup_path: self.spec()?.cgroup_path,
                            systemd_cgroup: self.systemd(),
                            container_name: self.id().to_string(),
                        },
                    )?;
                    cmanager.freeze(libcgroups::common::FreezerState::Thawed)?;
                }
                libcgroups::common::CgroupSetup::Unified => {}
            }
        }
        Ok(())
    }

    fn kill_all_processes<S: Into<Signal>>(&self, signal: S) -> Result<(), LibcontainerError> {
        let signal = signal.into().into_raw();
        let cmanager =
            libcgroups::common::create_cgroup_manager(libcgroups::common::CgroupConfig {
                cgroup_path: self.spec()?.cgroup_path,
                systemd_cgroup: self.systemd(),
                container_name: self.id().to_string(),
            })?;

        if let Err(e) = cmanager.freeze(libcgroups::common::FreezerState::Frozen) {
            tracing::warn!(
                err = ?e,
                id = ?self.id(),
                "failed to freeze container",
            );
        }

        let pids = cmanager.get_all_pids()?;
        pids.iter()
            .try_for_each(|&pid| {
                tracing::debug!("kill signal {} to {}", signal, pid);
                let rustix_pid = rustix::process::Pid::from_raw(pid.as_raw()).ok_or(nix::errno::Errno::EINVAL)?;
                let res = match rustix::process::pidfd_open(rustix_pid, rustix::process::PidfdFlags::empty()) {
                    Ok(fd) => {
                        // Recheck membership after acquiring the stable process handle.
                        if !cmanager.get_all_pids().map_err(|_| nix::errno::Errno::EIO)?.contains(&pid) { return Ok(()); }
                        let signal = rustix::process::Signal::from_named_raw(signal as i32).ok_or(nix::errno::Errno::EINVAL)?;
                        rustix::process::pidfd_send_signal(fd, signal).map_err(|err| nix::errno::Errno::from_raw(err.raw_os_error()))
                    }
                    Err(err) => Err(nix::errno::Errno::from_raw(err.raw_os_error())),
                };
                match res {
                    Err(nix::errno::Errno::ESRCH) => {
                        // the process does not exist, which is what we want
                        Ok(())
                    }
                    _ => res,
                }
            })
            .map_err(LibcontainerError::OtherSyscall)?;
        if let Err(err) = cmanager.freeze(libcgroups::common::FreezerState::Thawed) {
            tracing::warn!(
                err = ?err,
                id = ?self.id(),
                "failed to thaw container",
            );
        }

        Ok(())
    }
}
