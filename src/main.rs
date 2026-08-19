/// Minimal Hyperlight host for booting a Unikraft unikernel.
///
/// The guest boots, runs main(), prints via DebugPrint (port 103),
/// and halts (port 108).  Everything happens during evolve() —
/// no host functions or guest function calls are needed.
use std::env;

use hyperlight_host::{
    GuestBinary, MultiUseSandbox, UninitializedSandbox,
    sandbox::SandboxConfiguration,
};

fn main() -> hyperlight_host::Result<()> {
    let guest_path = env::args()
        .nth(1)
        .expect("Usage: hyperlight-unikraft-mini <guest-elf>");

    // Unikraft's native PAL allocates IST exception stacks in BSS
    // (CoW pages).  Each CoW page resolution consumes one scratch
    // page, and the default scratch size (0x48000 = 72 pages) isn't
    // enough.  Bump to 1 MiB to accommodate the ~50 IST pages plus
    // page tables and other BSS CoW faults.
    let mut cfg = SandboxConfiguration::default();
    cfg.set_scratch_size(0x100000); // 1 MiB
    cfg.set_heap_size(0x100000); // 1 MiB (default 128K too small for Unikraft stacks)

    let usandbox =
        UninitializedSandbox::new(GuestBinary::FilePath(guest_path), Some(cfg))?;

    // evolve() creates the VM and runs the guest to completion.
    // DebugPrint output (port 103) goes to stderr automatically.
    let _sandbox: MultiUseSandbox = usandbox.evolve()?;

    Ok(())
}
