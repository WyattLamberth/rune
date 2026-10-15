//! Native Rust Linux adapter. libcontainer owns kernel setup and persistent state.
use anyhow::{Result, bail};
use libcontainer::container::builder::ContainerBuilder;
use libcontainer::container::{Container, ContainerStatus};
use libcontainer::signal::Signal;
use libcontainer::syscall::syscall::SyscallType;
use liboci_cli::{Create, Delete, Kill};
use nix::fcntl::{Flock, FlockArg};
use rune_core::{Action, CommandError, Status, execute, valid_id};
use std::fs::{DirBuilder, File, OpenOptions};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

fn check_id(id: &str) -> Result<()> {
    if !valid_id(id) || id == ".rune-locks" {
        bail!("invalid container ID (use 1–128 ASCII letters, digits, '.', '_', '-')");
    }
    Ok(())
}
fn lock(root: &Path, id: &str) -> Result<Flock<File>> {
    check_id(id)?;
    DirBuilder::new().recursive(true).mode(0o700).create(root)?;
    let metadata = std::fs::symlink_metadata(root)?;
    if !metadata.is_dir()
        || metadata.uid() != nix::unistd::geteuid().as_raw()
        || metadata.permissions().mode() & 0o022 != 0
    {
        bail!(
            "runtime root must be a real directory owned by this UID, without group/world write permissions"
        );
    }
    std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o700))?;
    let locks = root.join(".rune-locks");
    DirBuilder::new().mode(0o700).create(&locks).or_else(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(e)
        }
    })?;
    let metadata = std::fs::symlink_metadata(&locks)?;
    if !metadata.is_dir()
        || metadata.uid() != nix::unistd::geteuid().as_raw()
        || metadata.permissions().mode() & 0o077 != 0
    {
        bail!("insecure runtime lock directory");
    }
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC)
        .open(locks.join(id))?;
    // Nonblocking: concurrent callers get a bounded, explicit busy error.
    Flock::lock(file, FlockArg::LockExclusiveNonblock)
        .map_err(|(_, err)| anyhow::anyhow!("container is busy: {err}"))
}
fn load(root: &Path, id: &str) -> Result<Container> {
    check_id(id)?;
    let container = Container::load(root.join(id))?;
    // A failed rollback must retain its single recovery marker and ownership state.
    // Native per-container locking prevents another acquisition while it exists.
    match std::fs::symlink_metadata(root.join(id).join("rune-pidfile-recovery.json")) {
        Ok(_) => bail!(
            "PID-file rollback ownership requires explicit recovery before lifecycle commands"
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if container.id() != id {
        bail!("stored container ID does not match requested ID");
    }
    Ok(container)
}
fn dispatch(
    container: &mut Container,
    action: Action,
    apply: impl FnOnce(&mut Container) -> Result<()>,
) -> Result<()> {
    let status = match container.status() {
        ContainerStatus::Creating => Status::Creating,
        ContainerStatus::Created => Status::Created,
        ContainerStatus::Running => Status::Running,
        ContainerStatus::Stopped => Status::Stopped,
        ContainerStatus::Paused => Status::Paused,
    };
    match execute(status, action, |_| apply(container)) {
        Ok(()) => Ok(()),
        Err(CommandError::InvalidTransition) => bail!("cannot {action:?} a {status:?} container"),
        Err(CommandError::Backend(error)) => Err(error),
    }
}
pub fn create(root: &Path, args: &Create, systemd: bool) -> Result<()> {
    let _lock = lock(root, &args.container_id)?;
    if args.no_new_keyring {
        bail!("--no-new-keyring is not supported by rune's current backend");
    }
    if !(0..=256).contains(&args.preserve_fds) {
        bail!("preserve-fds must be between 0 and 256");
    }
    let spec = crate::validation::bundle(&args.bundle)?;
    ContainerBuilder::new(args.container_id.clone(), SyscallType::default())
        .with_root_path(root)?
        .with_pid_file(args.pid_file.as_ref())?
        .with_console_socket(args.console_socket.as_ref())
        .with_preserved_fds(args.preserve_fds)
        .validate_id()?
        .as_init(&args.bundle)
        .with_spec(spec)
        .with_systemd(systemd)
        .with_detach(true)
        .with_no_pivot(args.no_pivot)
        .build()?;
    Ok(())
}
pub fn start(root: &Path, id: &str) -> Result<()> {
    let _lock = lock(root, id)?;
    let mut c = load(root, id)?;
    dispatch(&mut c, Action::Start, |c| {
        c.start()?;
        Ok(())
    })
}
pub fn state(root: &Path, id: &str) -> Result<()> {
    let _lock = lock(root, id)?;
    let c = load(root, id)?;
    // Emit only the OCI state contract; backend-specific storage fields stay private.
    let value = serde_json::json!({"ociVersion":c.state.oci_version,
        "id":c.id(), "status":c.status(), "pid":c.state.pid,
        "bundle":c.bundle(), "annotations":c.state.annotations});
    println!("{}", serde_json::to_string(&value)?);
    Ok(())
}
pub fn kill(root: &Path, args: &Kill) -> Result<()> {
    let _lock = lock(root, &args.container_id)?;
    let mut c = load(root, &args.container_id)?;
    dispatch(
        &mut c,
        if args.all {
            Action::KillAll
        } else {
            Action::Kill
        },
        |c| {
            c.kill(Signal::try_from(args.signal.as_str())?, args.all)?;
            Ok(())
        },
    )
}
pub fn delete(root: &Path, args: &Delete) -> Result<()> {
    let _lock = lock(root, &args.container_id)?;
    match std::fs::symlink_metadata(root.join(&args.container_id)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return match execute(
                Status::Absent,
                if args.force {
                    Action::ForceDelete
                } else {
                    Action::Delete
                },
                |_| -> Result<()> { bail!("absent deletion attempted a backend effect") },
            ) {
                Ok(()) => Ok(()),
                Err(CommandError::InvalidTransition) => bail!("container does not exist"),
                Err(CommandError::Backend(error)) => Err(error),
            };
        }
        Err(error) => return Err(error.into()),
        Ok(metadata) if !metadata.is_dir() => bail!("container state directory must be real"),
        Ok(_) => {}
    }
    let mut c = load(root, &args.container_id)?;
    dispatch(
        &mut c,
        if args.force {
            Action::ForceDelete
        } else {
            Action::Delete
        },
        |c| {
            c.delete(args.force)?;
            Ok(())
        },
    )
}
pub fn pause(root: &Path, id: &str) -> Result<()> {
    let _lock = lock(root, id)?;
    let mut c = load(root, id)?;
    dispatch(&mut c, Action::Pause, |c| {
        c.pause()?;
        Ok(())
    })
}
pub fn resume(root: &Path, id: &str) -> Result<()> {
    let _lock = lock(root, id)?;
    let mut c = load(root, id)?;
    dispatch(&mut c, Action::Resume, |c| {
        c.resume()?;
        Ok(())
    })
}

pub fn exec(root: &Path, args: &liboci_cli::Exec) -> Result<i32> {
    let held = lock(root, &args.container_id)?;
    let c = load(root, &args.container_id)?;
    if c.status() != ContainerStatus::Running {
        bail!("exec requires a running container");
    }
    if args.ignore_paused {
        bail!("--ignore-paused is not supported");
    }
    if args.tty && args.process.is_none() {
        bail!("direct exec --tty requires --process; Podman provides a process spec");
    }
    if !(0..=256).contains(&args.preserve_fds) {
        bail!("preserve-fds must be between 0 and 256");
    }
    let init = crate::validation::stored_process(&c.root)?;
    let supplied = args
        .process
        .as_deref()
        .map(crate::validation::process)
        .transpose()?;
    let process = crate::exec_policy::resolve(init, supplied, args)?;
    let pid = ContainerBuilder::new(args.container_id.clone(), SyscallType::default())
        .with_root_path(root)?
        .with_console_socket(args.console_socket.as_ref())
        .with_pid_file(args.pid_file.as_ref())?
        .with_preserved_fds(args.preserve_fds)
        .validate_id()?
        .as_tenant()
        .with_process_snapshot(Some(process))
        .with_detach(args.detach)
        .with_sub_cgroup(args.cgroup.clone())
        .build()?;
    drop(held);
    if args.detach {
        return Ok(0);
    }
    use nix::sys::wait::{WaitStatus, waitpid};
    match waitpid(pid, None)? {
        WaitStatus::Exited(_, code) => Ok(code),
        WaitStatus::Signaled(_, signal, _) => Ok(128 + signal as i32),
        _ => bail!("unexpected wait status for exec process"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locks_are_exclusive_and_survive_state_deletion() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let held = lock(dir.path(), "same")?;
        assert!(lock(dir.path(), "same").is_err());
        let other = lock(dir.path(), "other")?;
        drop(held);
        assert!(lock(dir.path(), "same").is_ok());
        drop(other);
        Ok(())
    }
    #[test]
    fn rejects_symlinked_root_and_path_escape() -> Result<()> {
        let dir = tempfile::tempdir()?;
        std::os::unix::fs::symlink(dir.path(), dir.path().join("link"))?;
        assert!(lock(&dir.path().join("link"), "test").is_err());
        assert!(lock(dir.path(), "../escape").is_err());
        assert!(lock(dir.path(), ".rune-locks").is_err());
        Ok(())
    }
}
