#![forbid(unsafe_code)]

mod exec_policy;
mod linux;
mod logging;
mod validation;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use liboci_cli::{Create, Delete, Exec, Kill, Pause, Resume, Start, State};
use logging::LogFormat;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "rune",
    version,
    about = "Rust OCI runtime (experimental, libcontainer backend)"
)]
struct Cli {
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[arg(long, global = true)]
    systemd_cgroup: bool,
    #[arg(long, global = true)]
    log: Option<PathBuf>,
    #[arg(long, global = true, default_value = "text", value_enum)]
    log_format: LogFormat,
    #[arg(long, global = true)]
    debug: bool,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand, Debug)]
enum Command {
    Create(Create),
    Start(Start),
    State(State),
    Kill(Kill),
    Delete(Delete),
    Pause(Pause),
    Resume(Resume),
    Exec(Box<Exec>),
}

fn root_path(requested: Option<&Path>) -> Result<PathBuf> {
    if let Some(root) = requested {
        return Ok(root.to_path_buf());
    }
    let Some(base) = std::env::var_os("XDG_RUNTIME_DIR") else {
        if nix::unistd::geteuid().is_root() {
            return Ok(PathBuf::from("/run/rune"));
        }
        bail!("set XDG_RUNTIME_DIR or pass --root");
    };
    let base = PathBuf::from(base);
    if !base.is_absolute() {
        bail!("XDG_RUNTIME_DIR must be absolute");
    }
    Ok(base.join("rune"))
}
fn execute(cli: &Cli) -> Result<i32> {
    let root = root_path(cli.root.as_deref())?;
    match &cli.command {
        Command::Create(args) => linux::create(&root, args, cli.systemd_cgroup),
        Command::Start(args) => linux::start(&root, &args.container_id),
        Command::State(args) => linux::state(&root, &args.container_id),
        Command::Kill(args) => linux::kill(&root, args),
        Command::Delete(args) => linux::delete(&root, args),
        Command::Pause(args) => linux::pause(&root, &args.container_id),
        Command::Resume(args) => linux::resume(&root, &args.container_id),
        Command::Exec(args) => return linux::exec(&root, args),
    }?;
    Ok(0)
}
fn run(cli: &Cli) -> i32 {
    let log = match logging::init(cli.log.as_deref(), cli.log_format, cli.debug) {
        Ok(log) => log,
        Err(error) => {
            eprintln!("rune: cannot initialize logging: {error:#}");
            return 1;
        }
    };
    let code = match execute(cli) {
        Ok(code) => code,
        Err(error) => {
            let message = format!("{error:#}");
            tracing::error!("{message}");
            if cli.log.is_some() {
                eprintln!("rune: {message}");
            }
            1
        }
    };
    if let Some(error) = log.failure() {
        eprintln!("rune: cannot write requested log: {error}");
        return if code == 0 { 1 } else { code };
    }
    code
}

fn main() {
    // run returns after synchronous log writes and drops its handles before exit.
    std::process::exit(run(&Cli::parse()));
}
