# hyperlight-unikraft-mini

Minimal Hyperlight host that boots a [Unikraft](https://unikraft.org/) unikernel.

## Prerequisites

- Rust toolchain (stable)
- Unikraft source tree with the `plat-hyperlight-cleanup` branch checked out
- A Unikraft application (e.g. the catalog `helloworld`)

## Build the guest

**1. Configure** — copy the defconfig into the build output directory and expand it:

```bash
mkdir -p /home/$USER/repos/hyperlight-unikraft-mini/build
cp /home/$USER/repos/hyperlight-unikraft-mini/defconfig \
   /home/$USER/repos/hyperlight-unikraft-mini/build/.config

make -C /home/$USER/repos/unikraft-project/unikraft \
    A=/home/$USER/repos/unikraft-project/catalog/library/helloworld \
    O=/home/$USER/repos/hyperlight-unikraft-mini/build \
    olddefconfig
```

`A=` tells Unikraft where the application source is (`Makefile.uk` + `main.c`).
`O=` is the build output directory.

**2. Build:**

```bash
make -C /home/$USER/repos/unikraft-project/unikraft \
    A=/home/$USER/repos/unikraft-project/catalog/library/helloworld \
    O=/home/$USER/repos/hyperlight-unikraft-mini/build \
    -j$(nproc)
```

The ELF binary lands at `build/helloworld_hyperlight-x86_64`.

## Run

```bash
cargo run -- build/helloworld_hyperlight-x86_64
```

Guest output (via Hyperlight's DebugPrint port) goes to stderr.
