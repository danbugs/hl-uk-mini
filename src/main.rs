use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

use hyperlight_unikraft::{
    Exec, DEFAULT_SCRATCH_MB, SNAPSHOT_TAG,
    create_sandbox, init, restore, run,
    OciTag, Snapshot,
};

/// Minimal Hyperlight host for Unikraft unikernels.
#[derive(Parser)]
#[command(name = "hl-uk-mini")]
struct Cli {
    /// Log level for hluk diagnostics: error, warn, info, debug, trace.
    /// Off by default; pass --log-level info to see timing.
    #[arg(long, global = true)]
    log_level: Option<tracing::Level>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Boot a Unikraft guest and dispatch exec commands.
    Run(RunArgs),

    /// Snapshot operations: save a post-evolve snapshot to disk,
    /// or restore from a saved snapshot and dispatch.
    #[command(subcommand)]
    Snapshot(SnapshotCommand),
}

#[derive(Subcommand)]
enum SnapshotCommand {
    /// Boot the guest, then save a snapshot to disk.
    Save(SaveArgs),

    /// Restore a guest from a saved snapshot and dispatch exec commands.
    Exec(ExecArgs),
}

/// Arguments for `run` — boot the embedded kernel + initrd and dispatch.
#[derive(clap::Args)]
struct RunArgs {
    /// Script file (.py, .js, …) to execute in the guest.
    #[arg(conflicts_with = "exec")]
    script: Option<PathBuf>,

    /// Path to a CPIO initrd to map into the guest.
    #[arg(long)]
    initrd: Option<PathBuf>,

    /// Entry point binary path inside the initrd VFS.
    /// Auto-detected from the initrd if not specified.
    #[arg(long)]
    entry: Option<String>,

    /// Scratch memory in MiB (default 256; increase for large rootfs).
    #[arg(long, default_value_t = DEFAULT_SCRATCH_MB)]
    scratch_mb: usize,

    /// Inline code to execute (alternative to a script file).
    #[arg(long, conflicts_with = "script")]
    exec: Option<String>,
}

/// Arguments for `snapshot save`.
#[derive(clap::Args)]
struct SaveArgs {
    /// Path to a CPIO initrd to map into the guest.
    #[arg(long)]
    initrd: Option<PathBuf>,

    /// Entry point binary path inside the initrd VFS.
    /// Auto-detected from the initrd if not specified.
    #[arg(long)]
    entry: Option<String>,

    /// Scratch memory in MiB (default 256; increase for large rootfs).
    #[arg(long, default_value_t = DEFAULT_SCRATCH_MB)]
    scratch_mb: usize,

    /// Directory to save the snapshot (OCI Image Layout).
    #[arg(short, long)]
    output: PathBuf,
}

/// Arguments for `snapshot exec`.
#[derive(clap::Args)]
struct ExecArgs {
    /// Path to a saved snapshot directory (OCI Image Layout).
    snapshot: PathBuf,

    /// Script file (.py, .js, …) to execute in the guest.
    #[arg(conflicts_with = "exec")]
    script: Option<PathBuf>,

    /// Inline code to execute (alternative to a script file).
    #[arg(long, conflicts_with = "script")]
    exec: Option<String>,
}

// ── Commands ─────────────────────────────────────────────────────

fn cmd_run(args: RunArgs) -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let (usandbox, _config) = create_sandbox(&args.initrd, &args.entry, args.scratch_mb)?;

    let t = Instant::now();
    let mut sandbox = init(usandbox)?;
    info!(elapsed_ms = t.elapsed().as_secs_f64() * 1000.0, "init");

    let exec = match (args.script, args.exec) {
        (Some(path), _) => Some(Exec::File(path)),
        (_, Some(code)) => Some(Exec::Code(code)),
        _ => None,
    };
    if let Some(exec) = exec {
        let t = Instant::now();
        run(&mut sandbox, exec)?;
        info!(elapsed_ms = t.elapsed().as_secs_f64() * 1000.0, "exec");
    }
    Ok(())
}

fn cmd_snapshot_save(args: SaveArgs) -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let (usandbox, _config) = create_sandbox(&args.initrd, &args.entry, args.scratch_mb)?;
    let mut sandbox = init(usandbox)?;

    let t = Instant::now();
    let snap = sandbox.snapshot()?;
    info!(
        elapsed_ms = t.elapsed().as_secs_f64() * 1000.0,
        "snapshot captured",
    );

    let tag: OciTag = SNAPSHOT_TAG.parse().expect("valid OCI tag");

    let t = Instant::now();
    let digest = snap.save(&args.output, &tag)?;
    info!(
        path = %args.output.display(),
        digest = %digest,
        elapsed_ms = t.elapsed().as_secs_f64() * 1000.0,
        "snapshot saved",
    );

    Ok(())
}

fn cmd_snapshot_exec(
    args: ExecArgs,
) -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let tag: OciTag = SNAPSHOT_TAG.parse().expect("valid OCI tag");

    let t = Instant::now();
    let snap: Arc<Snapshot> = Arc::new(Snapshot::load(&args.snapshot, tag)?);
    info!(
        path = %args.snapshot.display(),
        elapsed_ms = t.elapsed().as_secs_f64() * 1000.0,
        "snapshot loaded",
    );

    let t = Instant::now();
    let mut sandbox = restore(snap)?;
    info!(
        elapsed_ms = t.elapsed().as_secs_f64() * 1000.0,
        "restored from snapshot",
    );

    let exec = match (args.script, args.exec) {
        (Some(path), _) => Some(Exec::File(path)),
        (_, Some(code)) => Some(Exec::Code(code)),
        _ => None,
    };
    if let Some(exec) = exec {
        let t = Instant::now();
        run(&mut sandbox, exec)?;
        info!(elapsed_ms = t.elapsed().as_secs_f64() * 1000.0, "exec");
    }
    Ok(())
}

// ── Main ─────────────────────────────────────────────────────────

fn main() -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let cli = Cli::parse();

    if let Some(level) = cli.log_level {
        // RUST_LOG overrides --log-level when set; otherwise scope to
        // our crate only so library noise doesn't leak through.
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(format!("hyperlight_unikraft={level}")));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_writer(std::io::stderr)
            .init();
    }

    match cli.command {
        Command::Run(args) => cmd_run(args),
        Command::Snapshot(cmd) => match cmd {
            SnapshotCommand::Save(args) => cmd_snapshot_save(args),
            SnapshotCommand::Exec(args) => cmd_snapshot_exec(args),
        },
    }
}
