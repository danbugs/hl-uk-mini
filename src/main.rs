/// Minimal Hyperlight host for booting a Unikraft unikernel.
///
/// The guest boots, runs main(), prints via DebugPrint (port 103),
/// and halts (port 108).  Everything happens during evolve() —
/// no host functions or guest function calls are needed.
use std::env;

use hyperlight_host::{GuestBinary, MultiUseSandbox, UninitializedSandbox};

fn main() -> hyperlight_host::Result<()> {
    let guest_path = env::args().nth(1).expect("Usage: hyperlight-unikraft-mini <guest-elf>");

    let usandbox = UninitializedSandbox::new(GuestBinary::FilePath(guest_path), None)?;

    // evolve() creates the VM and runs the guest to completion.
    // DebugPrint output (port 103) goes to stderr automatically.
    let _sandbox: MultiUseSandbox = usandbox.evolve()?;

    Ok(())
}
