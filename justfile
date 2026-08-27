# hyperlight-unikraft (hluk)
#
# Cross-platform build recipes.
#
# Usage:
#   just build-rootfs python
#   just run python examples/python/hello.py
#   just clean
#
# TODO: add `just pull-rootfs <runtime>` to pull pre-built rootfs from registry
# TODO: Windows build-rootfs support (currently requires WSL)

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Directories
root_dir     := justfile_directory()
drivers_dir  := root_dir / "drivers"
build_dir    := root_dir / "build-elfloader"
snapshot_dir := root_dir / ".snapshots"
examples_dir := root_dir / "examples"

# Per-runtime scratch memory (MiB). Must cover rootfs extraction +
# runtime startup.
scratch_python := "256"
scratch_node   := "512"

# ── Build ────────────────────────────────────────────────────────

# Build the hluk CLI binary (release by default, pass `--debug` for debug)
[unix]
build *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    if echo "{{flags}}" | grep -q -- '--debug'; then
        cargo build --manifest-path "{{root_dir}}/Cargo.toml"
    else
        cargo build --release --manifest-path "{{root_dir}}/Cargo.toml"
    fi

[windows]
build *flags:
    if ("{{flags}}" -match '--debug') { cargo build --manifest-path "{{root_dir}}/Cargo.toml" } else { cargo build --release --manifest-path "{{root_dir}}/Cargo.toml" }

# ── Rootfs ───────────────────────────────────────────────────────

# Build a rootfs CPIO from a driver Dockerfile (e.g. just build-rootfs python)
[unix]
build-rootfs runtime:
    #!/usr/bin/env bash
    set -euo pipefail
    dockerfile="{{drivers_dir}}/{{runtime}}/Dockerfile"
    if [ ! -f "$dockerfile" ]; then
        echo "error: $dockerfile not found" >&2
        echo "available runtimes:" >&2
        ls -1 "{{drivers_dir}}" | while read f; do \
            [ -d "{{drivers_dir}}/$f" ] && echo "  $f"; \
        done >&2
        exit 1
    fi
    image="hluk-{{runtime}}-rootfs"
    output="{{build_dir}}/{{runtime}}-rootfs.cpio"
    mkdir -p "{{build_dir}}"
    echo "==> Building image $image from $dockerfile"
    docker build -t "$image" -f "$dockerfile" "{{drivers_dir}}/"
    echo "==> Exporting to $output (newc CPIO)"
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT
    cid=$(docker create --entrypoint=/ "$image" 2>/dev/null || docker create "$image")
    docker export "$cid" | tar -C "$tmpdir" -xf -
    docker rm "$cid" > /dev/null
    (cd "$tmpdir" && find . | cpio -o -H newc --quiet > "$output")
    echo "==> Done: $output ($(du -h "$output" | cut -f1))"

# TODO: Windows build-rootfs support
[windows]
build-rootfs runtime:
    @echo "error: build-rootfs requires WSL on Windows"
    @echo "Use: wsl just build-rootfs {{runtime}}"

# List available runtimes
[unix]
list-runtimes:
    @ls -1 "{{drivers_dir}}" | while read f; do \
        [ -d "{{drivers_dir}}/$f" ] && echo "$f"; \
    done

[windows]
list-runtimes:
    @Get-ChildItem -Directory "{{drivers_dir}}" | ForEach-Object { $_.Name }

# ── Run ──────────────────────────────────────────────────────────

# Build + run a script with a given runtime (e.g. just run python examples/python/hello.py)
[unix]
run runtime script *args:
    #!/usr/bin/env bash
    set -euo pipefail
    rootfs="{{build_dir}}/{{runtime}}-rootfs.cpio"
    if [ ! -f "$rootfs" ]; then
        echo "==> rootfs not found, building first..."
        just build-rootfs "{{runtime}}"
    fi
    # Look up per-runtime scratch size
    case "{{runtime}}" in
        python) scratch={{scratch_python}} ;;
        node)   scratch={{scratch_node}} ;;
        *)      scratch=256 ;;
    esac
    just build
    "{{root_dir}}/target/release/hyperlight-unikraft-mini" run \
        --initrd "$rootfs" \
        --scratch-mb "$scratch" \
        {{script}} {{args}}

# Run from a pre-built snapshot (e.g. just run-snapshot .snapshots/python hello.py)
[unix]
run-snapshot snapshot script *args:
    just build
    "{{root_dir}}/target/release/hyperlight-unikraft-mini" snapshot exec \
        {{snapshot}} {{script}} {{args}}

# ── Snapshot ─────────────────────────────────────────────────────

# Save a post-evolve snapshot (e.g. just snapshot-save python)
[unix]
snapshot-save runtime *args:
    #!/usr/bin/env bash
    set -euo pipefail
    rootfs="{{build_dir}}/{{runtime}}-rootfs.cpio"
    if [ ! -f "$rootfs" ]; then
        echo "==> rootfs not found, building first..."
        just build-rootfs "{{runtime}}"
    fi
    # Look up per-runtime scratch size
    case "{{runtime}}" in
        python) scratch={{scratch_python}} ;;
        node)   scratch={{scratch_node}} ;;
        *)      scratch=256 ;;
    esac
    just build
    mkdir -p "{{snapshot_dir}}"
    "{{root_dir}}/target/release/hyperlight-unikraft-mini" snapshot save \
        --initrd "$rootfs" \
        --scratch-mb "$scratch" \
        --output "{{snapshot_dir}}/{{runtime}}" {{args}}

# ── Examples ─────────────────────────────────────────────────────

# Run the Python hello world example
example-python: (run "python" (examples_dir / "python" / "hello.py"))

# Run the Node.js hello world example
example-node: (run "node" (examples_dir / "node" / "hello.js"))

# ── Test ─────────────────────────────────────────────────────────

# Run integration tests
test:
    cargo test --manifest-path "{{root_dir}}/Cargo.toml"

# Run benchmarks (TODO)
bench:
    @echo "TODO: coming later"

# Run upstream conformance tests for a runtime (TODO)
conformance runtime:
    @echo "TODO: coming later"

# ── Clean ────────────────────────────────────────────────────────

# Remove build artifacts
[unix]
clean:
    rm -rf "{{build_dir}}" "{{root_dir}}/target"

[windows]
clean:
    if (Test-Path "{{build_dir}}") { Remove-Item -Recurse -Force "{{build_dir}}" }
    if (Test-Path "{{root_dir}}/target") { Remove-Item -Recurse -Force "{{root_dir}}/target" }

# Remove only rootfs build artifacts (keep Rust build cache)
[unix]
clean-rootfs:
    rm -rf "{{build_dir}}"

[windows]
clean-rootfs:
    if (Test-Path "{{build_dir}}") { Remove-Item -Recurse -Force "{{build_dir}}" }
