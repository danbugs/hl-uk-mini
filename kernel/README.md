# Kernel

The embedded kernel binary (`elfloader_hyperlight-x86_64`) is a Unikraft
app-elfloader built against the `plat-hyperlight-cleanup` branch.

Users don't need to build this — it's embedded in the `hluk` binary via
`include_bytes!`. Only the rootfs (built with `just build-rootfs`) needs
to be produced by users.

## Configuration

See `../defconfig-elfloader` for the full kconfig used to build this kernel.

## Rebuilding from source

If you need to rebuild the kernel (e.g. to pick up Unikraft changes):

```bash
cd ~/repos/unikraft-project/app-elfloader

# Copy defconfig into the app-elfloader root (NOT build/.config — the
# Makefile always reads .config from here regardless of O= flags).
cp ~/repos/hyperlight-unikraft-mini/defconfig-elfloader .config

# Clean stale build artifacts, expand the defconfig, then build.
make properclean
make olddefconfig
make -j$(nproc)

# Copy the binary back
cp workdir/build/elfloader_hyperlight-x86_64 \
   ~/repos/hyperlight-unikraft-mini/kernel/
```

<!-- TODO: upstream kernel changes to kraft so this can be built with
     `kraft build --plat hyperlight --arch x86_64` without manual patching. -->
