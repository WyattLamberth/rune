#![forbid(unsafe_code)]

mod exec_policy;
mod linux;
mod validation;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use liboci_cli::{Create, Delete, Exec, Kill, Pause, Resume, Start, State};
use std::path::PathBuf;

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
    #[arg(long, global = true, default_value = "text", value_parser = ["text", "json"])]
    log_format: String,
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

fn root_path(requested: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(root) = requested {
        return Ok(root);
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
    let root = root_path(cli.root.clone())?;
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
fn main() {
    let cli = Cli::parse();
    if cli.debug {
        if let Some(path) = &cli.log {
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                Ok(file) => tracing_subscriber::fmt()
                    .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .init(),
                Err(err) => {
                    eprintln!("rune: cannot open debug log: {err}");
                    std::process::exit(1);
                }
            }
        } else {
            tracing_subscriber::fmt()
                .with_max_level(tracing_subscriber::filter::LevelFilter::WARN)
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .init();
        }
    }
    let result = execute(&cli);
    if let Ok(code) = result {
        std::process::exit(code);
    }
    if let Err(error) = result {
        let message = format!("{error:#}");
        eprintln!("rune: {message}");
        if let Some(path) = &cli.log {
            use std::io::Write;
            let line = if cli.log_format == "json" {
                serde_json::json!({"level":"error", "msg":message}).to_string()
            } else {
                message
            };
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .and_then(|mut file| writeln!(file, "{line}"))
            {
                Ok(()) => {}
                Err(err) => eprintln!("rune: cannot write requested error log: {err}"),
            }
        }
        std::process::exit(1);
    }
}
