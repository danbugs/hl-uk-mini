# hyperlight-unikraft (hluk)

[Hyperlight](https://github.com/hyperlight-dev/hyperlight) host for
[Unikraft](https://unikraft.org/) unikernels.

## Usage

```bash
# Build a Python rootfs and run a script
just build-rootfs python
just run python examples/python/hello.py

# Or Node.js
just build-rootfs node
just run node examples/node/hello.js

# Snapshots
just snapshot-save python
just run-snapshot .snapshots/python examples/python/hello.py

# See all recipes
just --list
```

## Supported runtimes

| Runtime | Status |
|---------|--------|
| Python  | ✅ CPython 3.12 (glibc) |
| Node.js | ✅ Node 21 (musl/Alpine) |
