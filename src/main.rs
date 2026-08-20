/// Minimal Hyperlight host for booting a Unikraft unikernel.
///
/// The guest boots, calls host functions for configuration (e.g.
/// GetCmdLine to fetch the command line), then halts (port 108).
use std::env;

use hyperlight_host::{
    GuestBinary, MultiUseSandbox, UninitializedSandbox,
    sandbox::SandboxConfiguration,
};

fn main() -> hyperlight_host::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hyperlight-unikraft-mini <guest-elf> [-- args...]");
        std::process::exit(1);
    }

    let guest_path = &args[1];

    // Build the command line for the guest.
    // Format: "unikraft-hyperlight [-- user-args...]"
    // If "--" is present, everything after it is forwarded as app args.
    let cmdline = {
        let sep_pos = args.iter().position(|a| a == "--");
        match sep_pos {
            Some(pos) => {
                let mut parts = vec!["unikraft-hyperlight".to_string()];
                parts.push("--".to_string());
                parts.extend_from_slice(&args[pos + 1..]);
                parts.join(" ")
            }
            None => "unikraft-hyperlight".to_string(),
        }
    };

    // Unikraft's native PAL allocates IST exception stacks in BSS
    // (CoW pages).  Each CoW page resolution consumes one scratch
    // page, and the default scratch size (0x48000 = 72 pages) isn't
    // enough.  Bump to 1 MiB to accommodate the ~50 IST pages plus
    // page tables and other BSS CoW faults.
    let mut cfg = SandboxConfiguration::default();
    cfg.set_scratch_size(0x100000); // 1 MiB
    cfg.set_heap_size(0x100000); // 1 MiB (default 128K too small for Unikraft stacks)

    let mut usandbox =
        UninitializedSandbox::new(GuestBinary::FilePath(guest_path.clone()), Some(cfg))?;

    // Register host functions that the guest can call during boot.

    // GetCmdLine() -> String: returns the command line for Unikraft.
    let cmdline_clone = cmdline.clone();
    usandbox.register("GetCmdLine", move || -> hyperlight_host::Result<String> {
        Ok(cmdline_clone.clone())
    })?;

    eprintln!("[host] cmdline: {cmdline}");

    // evolve() creates the VM and runs the guest to completion.
    // During evolve, the guest can call registered host functions
    // (e.g. GetCmdLine).  DebugPrint output goes to stderr.
    let _sandbox: MultiUseSandbox = usandbox.evolve()?;

    Ok(())
}
