mod host_functions;

use std::fs::File;
use std::io::{Read as _, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use clap::{Parser, Subcommand};
use hyperlight_host::{
    GuestBinary, MultiUseSandbox, UninitializedSandbox,
    sandbox::{SandboxConfiguration, snapshot::{OciTag, Snapshot}},
};

use host_functions::GuestConfig;

/// GPA where the initrd is mapped via map_file_cow.
/// Past the x86 LAPIC MMIO page (0xFEE0_0000) to avoid collisions
/// with KVM's in-kernel IRQCHIP reservation.
const INITRD_MAP_BASE: u64 = 0xFEF0_0000;

/// Default scratch memory budget.  The frame allocator gets 75% of
/// this; the rest covers CoW faults and boot overhead.  Override
/// with --scratch-mb for large rootfs images (e.g. Node's 100 MiB
/// binary needs ~512 MiB).
const DEFAULT_SCRATCH_MB: usize = 256;

/// PEB heap size.  Only needed for the boot stack (allocated before
/// ukplat_mem_init).  Can be dropped to 0 once the guest allocates
/// the boot stack from scratch instead.
const HEAP_SIZE: u64 = 0x10_0000; // 1 MiB

/// OCI tag used when saving/loading snapshots to disk.
const SNAPSHOT_TAG: &str = "latest";

/// Minimal Hyperlight host for Unikraft unikernels.
#[derive(Parser)]
#[command(name = "hl-uk-mini")]
struct Cli {
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

/// Arguments shared by commands that boot from an ELF binary.
#[derive(clap::Args)]
struct RunArgs {
    /// Path to the guest ELF binary.
    guest_elf: PathBuf,

    /// Script file (.py, .js, …) to execute in the guest.
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

    /// Inline code to dispatch (repeatable).  If a script file is
    /// also given, --exec runs after the script.
    #[arg(long)]
    exec: Vec<String>,
}

/// Arguments for `snapshot save`.
#[derive(clap::Args)]
struct SaveArgs {
    /// Path to the guest ELF binary.
    guest_elf: PathBuf,

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
    script: Option<PathBuf>,

    /// Inline code to dispatch (repeatable).  If a script file is
    /// also given, --exec runs after the script.
    #[arg(long)]
    exec: Vec<String>,
}

// ── CPIO entry-point auto-detection ─────────────────────────────

/// Scan a newc-format CPIO archive for a Hyperlight driver binary.
///
/// Looks for files matching `usr/local/bin/hl_*` or `usr/bin/hl_*`
/// and returns the first match as a VFS-absolute path (e.g.
/// `/usr/local/bin/hl_pydriver`).
fn find_cpio_entry(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut header = [0u8; 110];

    loop {
        if file.read_exact(&mut header).is_err() {
            break;
        }

        let magic = std::str::from_utf8(&header[0..6]).ok()?;
        if magic != "070701" && magic != "070702" {
            break;
        }

        let namesize =
            u32::from_str_radix(std::str::from_utf8(&header[94..102]).ok()?, 16).ok()?;
        let filesize =
            u64::from_str_radix(std::str::from_utf8(&header[54..62]).ok()?, 16).ok()?;

        let mut name_buf = vec![0u8; namesize as usize];
        file.read_exact(&mut name_buf).ok()?;
        let name = std::str::from_utf8(&name_buf)
            .ok()?
            .trim_end_matches('\0');

        if name == "TRAILER!!!" {
            break;
        }

        // Pad past filename to 4-byte boundary
        let name_padding = (4 - ((110 + namesize) % 4)) % 4;
        file.seek(SeekFrom::Current(name_padding as i64)).ok()?;

        // Check for a Hyperlight driver binary
        if name.starts_with("usr/local/bin/hl_") || name.starts_with("usr/bin/hl_") {
            return Some(format!("/{name}"));
        }

        // Skip file data + padding to 4-byte boundary
        let data_padding = (4 - (filesize % 4)) % 4;
        file.seek(SeekFrom::Current((filesize + data_padding) as i64))
            .ok()?;
    }

    None
}

/// Resolve the entry point: explicit --entry, auto-detected from
/// the initrd, or None (kernel boots without loading an app).
fn resolve_entry(entry: &Option<String>, initrd: &Option<PathBuf>) -> Option<String> {
    if let Some(e) = entry {
        return Some(e.clone());
    }
    if let Some(path) = initrd {
        if let Some(detected) = find_cpio_entry(path) {
            eprintln!("[host] auto-detected entry: {detected}");
            return Some(detected);
        }
    }
    None
}

// ── Helpers ──────────────────────────────────────────────────────

/// Build the guest command line from the entry point.
fn build_cmdline(entry: &Option<String>) -> String {
    match entry {
        Some(e) => format!("unikraft-hyperlight {e}"),
        None => "unikraft-hyperlight".to_string(),
    }
}

/// Create an uninitialized sandbox with host functions registered.
fn create_sandbox(
    guest_elf: &PathBuf,
    initrd: &Option<PathBuf>,
    entry: &Option<String>,
    scratch_mb: usize,
) -> hyperlight_host::Result<(UninitializedSandbox, GuestConfig)> {
    let scratch_size = scratch_mb * 1024 * 1024;
    let mut cfg = SandboxConfiguration::default();
    cfg.set_scratch_size(scratch_size);
    cfg.set_heap_size(HEAP_SIZE);

    let mut usandbox = UninitializedSandbox::new(
        GuestBinary::FilePath(guest_elf.display().to_string()),
        Some(cfg),
    )?;

    let (initrd_base, initrd_size) = if let Some(path) = initrd {
        let size = usandbox.map_file_cow(path, INITRD_MAP_BASE)?;
        eprintln!(
            "[host] initrd: {} ({size} bytes) mapped at GPA {INITRD_MAP_BASE:#x}",
            path.display(),
        );
        (INITRD_MAP_BASE, size)
    } else {
        (0, 0)
    };

    let entry = resolve_entry(entry, initrd);
    let config = GuestConfig {
        cmdline: build_cmdline(&entry),
        scratch_size,
        initrd_base,
        initrd_size,
    };

    config.register(&mut usandbox)?;
    eprintln!("[host] cmdline: {}", config.cmdline);

    Ok((usandbox, config))
}

/// Evolve a sandbox and print timing.
fn evolve(usandbox: UninitializedSandbox) -> hyperlight_host::Result<MultiUseSandbox> {
    let t = Instant::now();
    let sandbox = usandbox.evolve()?;
    eprintln!("[host] evolve: {:.1}ms", t.elapsed().as_secs_f64() * 1000.0);
    Ok(sandbox)
}

/// Build the list of code strings to dispatch: script file first
/// (read into a string), then any --exec inline snippets.
fn resolve_exec(script: &Option<PathBuf>, exec: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(path) = script {
        match std::fs::read_to_string(path) {
            Ok(code) => out.push(code),
            Err(e) => {
                eprintln!("[host] error reading {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    }
    out.extend(exec.iter().cloned());
    out
}

/// Dispatch exec commands on a sandbox and print timing.
fn dispatch(
    sandbox: &mut MultiUseSandbox,
    script: &Option<PathBuf>,
    exec: &[String],
    label: &str,
) -> hyperlight_host::Result<()> {
    let items = resolve_exec(script, exec);
    for (i, code) in items.iter().enumerate() {
        let summary = if script.is_some() && i == 0 {
            format!("{}", script.as_ref().unwrap().display())
        } else {
            code.chars().take(60).collect()
        };
        let t = Instant::now();
        sandbox.call::<()>("Exec", code.clone())?;
        eprintln!(
            "[host] exec[{i}]{label}: {:.1}ms ({summary})",
            t.elapsed().as_secs_f64() * 1000.0,
        );
    }
    Ok(())
}

// ── Commands ─────────────────────────────────────────────────────

fn cmd_run(args: RunArgs) -> hyperlight_host::Result<()> {
    let (usandbox, _config) = create_sandbox(&args.guest_elf, &args.initrd, &args.entry, args.scratch_mb)?;
    let mut sandbox = evolve(usandbox)?;
    dispatch(&mut sandbox, &args.script, &args.exec, "")?;
    Ok(())
}

fn cmd_snapshot_save(args: SaveArgs) -> hyperlight_host::Result<()> {
    let (usandbox, _config) = create_sandbox(&args.guest_elf, &args.initrd, &args.entry, args.scratch_mb)?;
    let mut sandbox = evolve(usandbox)?;

    let t = Instant::now();
    let snap = sandbox.snapshot()?;
    eprintln!(
        "[host] snapshot: {:.1}ms",
        t.elapsed().as_secs_f64() * 1000.0,
    );

    let tag: OciTag = SNAPSHOT_TAG.parse().expect("valid OCI tag");

    let t = Instant::now();
    let digest = snap.save(&args.output, &tag)?;
    eprintln!(
        "[host] saved: {} ({digest}) in {:.1}ms",
        args.output.display(),
        t.elapsed().as_secs_f64() * 1000.0,
    );

    Ok(())
}

fn cmd_snapshot_exec(args: ExecArgs) -> hyperlight_host::Result<()> {
    let tag: OciTag = SNAPSHOT_TAG.parse().expect("valid OCI tag");

    let t = Instant::now();
    let snap: Arc<Snapshot> = Arc::new(Snapshot::load(&args.snapshot, tag)?);
    eprintln!(
        "[host] loaded snapshot: {} in {:.1}ms",
        args.snapshot.display(),
        t.elapsed().as_secs_f64() * 1000.0,
    );

    // Build host functions.  The snapshot validates that we provide
    // a superset of what was registered at save time.  The guest
    // already has the real values baked into its memory from evolve,
    // so safe defaults suffice here.
    let config = GuestConfig {
        cmdline: String::new(),
        scratch_size: DEFAULT_SCRATCH_MB * 1024 * 1024,
        initrd_base: 0,
        initrd_size: 0,
    };
    let hf = config.host_functions()?;

    let t = Instant::now();
    let mut sandbox = MultiUseSandbox::from_snapshot(snap, hf, None)?;
    eprintln!(
        "[host] from_snapshot: {:.1}ms",
        t.elapsed().as_secs_f64() * 1000.0,
    );

    dispatch(&mut sandbox, &args.script, &args.exec, " (from snapshot)")?;
    Ok(())
}

// ── Main ─────────────────────────────────────────────────────────

fn main() -> hyperlight_host::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run(args) => cmd_run(args),
        Command::Snapshot(cmd) => match cmd {
            SnapshotCommand::Save(args) => cmd_snapshot_save(args),
            SnapshotCommand::Exec(args) => cmd_snapshot_exec(args),
        },
    }
}
