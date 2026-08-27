use std::path::PathBuf;
use std::sync::{Arc, Barrier};
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

    /// Benchmark modes with structured timing output.
    #[command(subcommand)]
    Bench(BenchCommand),
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

#[derive(Subcommand)]
enum BenchCommand {
    /// Cold start: fresh boot (evolve) + dispatch, no snapshot.
    Cold(BenchColdArgs),

    /// Cold snapshot start: load snapshot from disk + restore + dispatch.
    ColdSnap(BenchSnapArgs),

    /// Warm with restore: load snapshot once, then loop (dispatch + restore).
    WarmRestore(BenchSnapArgs),

    /// Warm stateful: load snapshot once, then loop (dispatch only, no restore).
    WarmStateful(BenchSnapArgs),

    /// Parallel VMs: spawn N VMs concurrently from the same snapshot.
    Parallel(BenchParallelArgs),
}

/// Arguments for `bench cold`.
#[derive(clap::Args)]
struct BenchColdArgs {
    /// Path to a CPIO initrd.
    #[arg(long)]
    initrd: PathBuf,

    /// Script file to execute.
    script: PathBuf,

    /// Scratch memory in MiB.
    #[arg(long, default_value_t = DEFAULT_SCRATCH_MB)]
    scratch_mb: usize,

    /// Number of samples to run.
    #[arg(long, default_value_t = 20)]
    samples: usize,
}

/// Arguments for snapshot-based bench modes (cold-snap, warm-restore, warm-stateful).
#[derive(clap::Args)]
struct BenchSnapArgs {
    /// Path to a saved snapshot directory.
    snapshot: PathBuf,

    /// Script file to execute.
    script: PathBuf,

    /// Number of iterations / samples.
    #[arg(long, default_value_t = 20)]
    samples: usize,
}

/// Arguments for `bench parallel`.
#[derive(clap::Args)]
struct BenchParallelArgs {
    /// Path to a saved snapshot directory.
    snapshot: PathBuf,

    /// Script file to execute.
    script: PathBuf,

    /// Number of concurrent VMs.
    #[arg(long, default_value_t = 4)]
    vms: usize,

    /// Iterations per VM.
    #[arg(long, default_value_t = 10)]
    iterations: usize,
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

// ── Bench helpers ────────────────────────────────────────────────

fn read_script(path: &std::path::Path) -> hyperlight_unikraft::hyperlight_host::Result<String> {
    std::fs::read_to_string(path).map_err(|e| {
        hyperlight_unikraft::hyperlight_host::HyperlightError::Error(format!(
            "failed to read {}: {e}",
            path.display(),
        ))
    })
}

/// Percentile from a **sorted** slice (linear interpolation).
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = p / 100.0 * (sorted.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

/// Print summary line: median, p95, min, max.
fn print_summary(label: &str, field: &str, values: &[f64]) {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = percentile(&sorted, 50.0);
    let p95 = percentile(&sorted, 95.0);
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    println!(
        "BENCH {label} {field} median={median:.3} p95={p95:.3} min={min:.3} max={max:.3} samples={}",
        values.len(),
    );
}

/// Print snapshot size on disk as a BENCH line.
fn print_snapshot_size(label: &str, snap_dir: &std::path::Path) {
    fn dir_size(path: &std::path::Path) -> std::io::Result<u64> {
        let mut total = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_file() {
                total += meta.len();
            } else if meta.is_dir() {
                total += dir_size(&entry.path())?;
            }
        }
        Ok(total)
    }
    if let Ok(bytes) = dir_size(snap_dir) {
        let mib = bytes as f64 / (1024.0 * 1024.0);
        println!("BENCH {label} snapshot_mib={mib:.1}");
    }
}

/// Print private (anonymous) RSS as a BENCH line.
/// This is the density-relevant metric — it scales linearly with VM count.
/// Closest analog to Windows' PrivateMemorySize64.
fn print_rss(label: &str) {
    #[cfg(target_os = "linux")]
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        if let Some(kb) = status
            .lines()
            .find(|l| l.starts_with("RssAnon:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u64>().ok())
        {
            println!("BENCH {label} rss_mb={}", kb / 1024);
        }
    }
    // TODO: Windows — use GetProcessMemoryInfo for PrivateUsage
    #[cfg(not(target_os = "linux"))]
    let _ = label;
}

// ── Bench commands ───────────────────────────────────────────────

/// Cold start: fresh boot + dispatch, N independent samples.
fn bench_cold(args: BenchColdArgs) -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let source = read_script(&args.script)?;
    let mut boots = Vec::with_capacity(args.samples);
    let mut execs = Vec::with_capacity(args.samples);
    let mut totals = Vec::with_capacity(args.samples);

    for i in 0..args.samples {
        let t0 = Instant::now();
        let (usandbox, _) = create_sandbox(&Some(args.initrd.clone()), &None, args.scratch_mb)?;
        let mut sandbox = init(usandbox)?;
        let boot_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let t1 = Instant::now();
        run(&mut sandbox, source.as_str())?;
        let exec_ms = t1.elapsed().as_secs_f64() * 1000.0;

        let total_ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!(
            "BENCH cold sample={i} boot_ms={boot_ms:.3} exec_ms={exec_ms:.3} total_ms={total_ms:.3}"
        );
        boots.push(boot_ms);
        execs.push(exec_ms);
        totals.push(total_ms);
    }

    print_summary("cold", "boot_ms", &boots);
    print_summary("cold", "exec_ms", &execs);
    print_summary("cold", "total_ms", &totals);
    print_rss("cold");
    Ok(())
}

/// Cold snapshot: load from disk + restore + dispatch, N independent samples.
fn bench_cold_snap(args: BenchSnapArgs) -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let source = read_script(&args.script)?;
    let tag: OciTag = SNAPSHOT_TAG.parse().expect("valid OCI tag");
    let mut loads = Vec::with_capacity(args.samples);
    let mut restores = Vec::with_capacity(args.samples);
    let mut execs = Vec::with_capacity(args.samples);
    let mut totals = Vec::with_capacity(args.samples);

    for i in 0..args.samples {
        let t0 = Instant::now();
        let snap = Arc::new(Snapshot::load(&args.snapshot, tag.clone())?);
        let load_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let t1 = Instant::now();
        let mut sandbox = restore(snap)?;
        let restore_ms = t1.elapsed().as_secs_f64() * 1000.0;

        let t2 = Instant::now();
        run(&mut sandbox, source.as_str())?;
        let exec_ms = t2.elapsed().as_secs_f64() * 1000.0;

        let total_ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!(
            "BENCH cold-snap sample={i} load_ms={load_ms:.3} restore_ms={restore_ms:.3} \
             exec_ms={exec_ms:.3} total_ms={total_ms:.3}"
        );
        loads.push(load_ms);
        restores.push(restore_ms);
        execs.push(exec_ms);
        totals.push(total_ms);
    }

    print_summary("cold-snap", "load_ms", &loads);
    print_summary("cold-snap", "restore_ms", &restores);
    print_summary("cold-snap", "exec_ms", &execs);
    print_summary("cold-snap", "total_ms", &totals);
    print_snapshot_size("cold-snap", &args.snapshot);
    print_rss("cold-snap");
    Ok(())
}

/// Warm with restore: load snapshot once, then loop dispatch + restore.
fn bench_warm_restore(args: BenchSnapArgs) -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let source = read_script(&args.script)?;
    let tag: OciTag = SNAPSHOT_TAG.parse().expect("valid OCI tag");

    let t0 = Instant::now();
    let snap = Arc::new(Snapshot::load(&args.snapshot, tag)?);
    let mut sandbox = restore(snap.clone())?;
    let setup_ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("BENCH warm-restore setup_ms={setup_ms:.3}");

    let mut execs = Vec::with_capacity(args.samples);
    let mut restores = Vec::with_capacity(args.samples);

    for i in 0..args.samples {
        let t1 = Instant::now();
        run(&mut sandbox, source.as_str())?;
        let exec_ms = t1.elapsed().as_secs_f64() * 1000.0;

        let t2 = Instant::now();
        sandbox.restore(snap.clone())?;
        let restore_ms = t2.elapsed().as_secs_f64() * 1000.0;

        println!(
            "BENCH warm-restore sample={i} exec_ms={exec_ms:.3} restore_ms={restore_ms:.3}"
        );
        execs.push(exec_ms);
        restores.push(restore_ms);
    }

    print_summary("warm-restore", "exec_ms", &execs);
    print_summary("warm-restore", "restore_ms", &restores);
    print_snapshot_size("warm-restore", &args.snapshot);
    print_rss("warm-restore");
    Ok(())
}

/// Warm stateful: load snapshot once, then loop dispatch without restore.
fn bench_warm_stateful(args: BenchSnapArgs) -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let source = read_script(&args.script)?;
    let tag: OciTag = SNAPSHOT_TAG.parse().expect("valid OCI tag");

    let t0 = Instant::now();
    let snap = Arc::new(Snapshot::load(&args.snapshot, tag)?);
    let mut sandbox = restore(snap)?;
    let setup_ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("BENCH warm-stateful setup_ms={setup_ms:.3}");

    let mut execs = Vec::with_capacity(args.samples);

    for i in 0..args.samples {
        let t1 = Instant::now();
        run(&mut sandbox, source.as_str())?;
        let exec_ms = t1.elapsed().as_secs_f64() * 1000.0;

        println!("BENCH warm-stateful sample={i} exec_ms={exec_ms:.3}");
        execs.push(exec_ms);
    }

    print_summary("warm-stateful", "exec_ms", &execs);
    print_snapshot_size("warm-stateful", &args.snapshot);
    print_rss("warm-stateful");
    Ok(())
}

/// Parallel VMs: spawn N threads, each restoring from the same snapshot.
fn bench_parallel(args: BenchParallelArgs) -> hyperlight_unikraft::hyperlight_host::Result<()> {
    let source = Arc::new(read_script(&args.script)?);
    let tag: OciTag = SNAPSHOT_TAG.parse().expect("valid OCI tag");
    let snap = Arc::new(Snapshot::load(&args.snapshot, tag)?);

    // Barrier so all VMs start at the same time.
    let barrier = Arc::new(Barrier::new(args.vms));
    let wall_start = Instant::now();

    let handles: Vec<_> = (0..args.vms)
        .map(|vm_id| {
            let snap = snap.clone();
            let source = source.clone();
            let barrier = barrier.clone();
            let iterations = args.iterations;

            std::thread::spawn(move || -> Result<Vec<f64>, String> {
                barrier.wait();
                let vm_start = Instant::now();

                let mut sandbox = restore(snap.clone()).map_err(|e| e.to_string())?;
                let mut execs = Vec::with_capacity(iterations);

                for iter in 0..iterations {
                    let t = Instant::now();
                    run(&mut sandbox, source.as_str()).map_err(|e| e.to_string())?;
                    let exec_ms = t.elapsed().as_secs_f64() * 1000.0;

                    let t = Instant::now();
                    sandbox.restore(snap.clone()).map_err(|e| e.to_string())?;
                    let restore_ms = t.elapsed().as_secs_f64() * 1000.0;

                    println!(
                        "BENCH parallel vm={vm_id} iter={iter} \
                         exec_ms={exec_ms:.3} restore_ms={restore_ms:.3}"
                    );
                    execs.push(exec_ms);
                }

                let vm_total_ms = vm_start.elapsed().as_secs_f64() * 1000.0;
                let vm_throughput = iterations as f64 / (vm_total_ms / 1000.0);
                println!(
                    "BENCH parallel vm={vm_id} total_ms={vm_total_ms:.3} \
                     iterations={iterations} throughput={vm_throughput:.1}/s"
                );
                Ok(execs)
            })
        })
        .collect();

    let mut errors = Vec::new();
    let mut all_execs = Vec::new();
    for (i, h) in handles.into_iter().enumerate() {
        match h.join().unwrap_or_else(|_| Err("thread panicked".into())) {
            Ok(execs) => all_execs.extend(execs),
            Err(e) => errors.push(format!("vm {i}: {e}")),
        }
    }

    let wall_ms = wall_start.elapsed().as_secs_f64() * 1000.0;
    let total_calls = args.vms * args.iterations;
    let throughput = total_calls as f64 / (wall_ms / 1000.0);
    println!(
        "BENCH parallel summary vms={} iterations={} total_calls={total_calls} \
         wall_ms={wall_ms:.3} throughput={throughput:.1}/s",
        args.vms, args.iterations,
    );
    if !all_execs.is_empty() {
        print_summary("parallel", "exec_ms", &all_execs);
    }
    print_snapshot_size("parallel", &args.snapshot);
    print_rss("parallel");

    if !errors.is_empty() {
        return Err(hyperlight_unikraft::hyperlight_host::HyperlightError::Error(
            errors.join("; "),
        ));
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
        Command::Bench(cmd) => match cmd {
            BenchCommand::Cold(args) => bench_cold(args),
            BenchCommand::ColdSnap(args) => bench_cold_snap(args),
            BenchCommand::WarmRestore(args) => bench_warm_restore(args),
            BenchCommand::WarmStateful(args) => bench_warm_stateful(args),
            BenchCommand::Parallel(args) => bench_parallel(args),
        },
    }
}
