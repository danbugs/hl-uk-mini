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
root_dir        := justfile_directory()
drivers_dir     := root_dir / "drivers"
build_dir       := root_dir / "build-elfloader"
snapshot_dir    := root_dir / ".snapshots"
examples_dir    := root_dir / "examples"
conformance_dir := root_dir / "conformance"

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
    "{{root_dir}}/target/release/hyperlight-unikraft" run \
        --initrd "$rootfs" \
        --scratch-mb "$scratch" \
        {{script}} {{args}}

# Run from a pre-built snapshot (e.g. just run-snapshot .snapshots/python hello.py)
[unix]
run-snapshot snapshot script *args:
    just build
    "{{root_dir}}/target/release/hyperlight-unikraft" snapshot exec \
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
    "{{root_dir}}/target/release/hyperlight-unikraft" snapshot save \
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

# ── Conformance ─────────────────────────────────────────────────

# Build a conformance rootfs (includes upstream test suite).
# Depends on the base driver image — builds it first if missing.
[unix]
build-conformance runtime:
    #!/usr/bin/env bash
    set -euo pipefail
    dockerfile="{{conformance_dir}}/{{runtime}}/Dockerfile"
    if [ ! -f "$dockerfile" ]; then
        echo "error: $dockerfile not found" >&2
        exit 1
    fi
    # Ensure the base driver image exists
    base_image="hluk-{{runtime}}-rootfs"
    if ! docker image inspect "$base_image" >/dev/null 2>&1; then
        echo "==> Base image $base_image not found, building first..."
        just build-rootfs "{{runtime}}"
    fi
    image="hluk-{{runtime}}-conformance"
    output="{{build_dir}}/{{runtime}}-conformance.cpio"
    mkdir -p "{{build_dir}}"
    echo "==> Building conformance image $image"
    docker build -t "$image" -f "$dockerfile" "{{conformance_dir}}/{{runtime}}/"
    echo "==> Exporting to $output (newc CPIO)"
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT
    cid=$(docker create --entrypoint=/ "$image" 2>/dev/null || docker create "$image")
    docker export "$cid" | tar -C "$tmpdir" -xf -
    docker rm "$cid" > /dev/null
    (cd "$tmpdir" && find . | cpio -o -H newc --quiet > "$output")
    echo "==> Done: $output ($(du -h "$output" | cut -f1))"

# Run upstream conformance tests for a runtime.
# Each test module runs in its own guest (snapshot restore) so a crash
# in one doesn't kill the suite and memory resets between tests.
#
# Modules listed in conformance/<runtime>/known_failures.toml are
# skipped — any module NOT in that file is expected to pass.
[unix]
conformance runtime *modules:
    #!/usr/bin/env bash
    set -uo pipefail
    hluk="{{root_dir}}/target/release/hyperlight-unikraft"
    rootfs="{{build_dir}}/{{runtime}}-conformance.cpio"
    snap_dir="{{snapshot_dir}}/{{runtime}}-conformance"
    manifest="{{conformance_dir}}/{{runtime}}/known_failures.toml"

    if [ ! -f "$rootfs" ]; then
        echo "==> Conformance rootfs not found, building first..."
        just build-conformance "{{runtime}}"
    fi

    case "{{runtime}}" in
        python) scratch={{scratch_python}} ;;
        node)   scratch={{scratch_node}} ;;
        *)      scratch=256 ;;
    esac

    just build

    # Save a snapshot if one doesn't exist
    if [ ! -d "$snap_dir" ]; then
        echo "==> Saving conformance snapshot..."
        "$hluk" snapshot save \
            --initrd "$rootfs" \
            --scratch-mb "$scratch" \
            --output "$snap_dir"
    fi

    # Determine which modules to test.  If args given, use those;
    # otherwise discover test_*.py modules from the rootfs via guest.
    if [ -n "{{modules}}" ]; then
        test_modules=({{modules}})
    else
        echo "==> Discovering test modules..."
        # One module per line to avoid output truncation
        mapfile -t test_modules < <("$hluk" snapshot exec "$snap_dir" --exec \
            "import os; d='/usr/local/lib/python3.12/test'; [print(f[:-3]) for f in sorted(os.listdir(d)) if f.startswith('test_') and f.endswith('.py')]" \
            2>/dev/null | tr -d '\r' | grep "^test_" || true)
    fi

    # Build skip list from the known-failures manifest
    declare -A SKIP=()
    if [ -f "$manifest" ]; then
        while IFS= read -r mod; do
            SKIP["$mod"]=1
        done < <(grep -oP '^\s+"(test_[^"]+)"' "$manifest" | sed 's/.*"\(test_[^"]*\)".*/\1/')
    fi

    pass=0 fail=0 error=0 skip=0 crash=0 total=0

    echo "==> Running ${#test_modules[@]} modules (${#SKIP[@]} in skip list)"
    echo ""

    for mod in "${test_modules[@]}"; do
        if [[ -n "${SKIP[$mod]+x}" ]]; then
            echo "SKIP $mod"
            ((skip++)) || true
            ((total++)) || true
            continue
        fi

        # Run this module in its own guest with a timeout.
        # --kill-after=5 ensures a SIGKILL follows if the process
        # ignores SIGTERM (some modules spin the hypervisor at 100% CPU).
        inline=$(printf "MODULE='%s'\n%s" "$mod" "$(cat '{{conformance_dir}}/{{runtime}}/run_tests.py')")

        output=$(timeout --kill-after=5 60 "$hluk" snapshot exec "$snap_dir" \
            --exec "$inline" 2>/dev/null || echo "RESULT $mod status=CRASH tests=0 fail=0 error=0 skip=0 time=0")

        # Extract the RESULT line
        result_line=$(echo "$output" | tr -d '\r' | grep "^RESULT " | tail -1)

        if [ -z "$result_line" ]; then
            result_line="RESULT $mod status=CRASH tests=0 fail=0 error=0 skip=0 time=0"
        fi

        echo "$result_line"

        status=$(echo "$result_line" | grep -oP 'status=\K\w+')
        case "$status" in
            PASS)  ((pass++))  || true ;;
            FAIL)  ((fail++))  || true ;;
            ERROR) ((error++)) || true ;;
            CRASH) ((crash++)) || true ;;
        esac
        ((total++)) || true
    done

    echo ""
    echo "════════════════════════════════════════════"
    echo "SUMMARY total=$total pass=$pass fail=$fail error=$error skip=$skip crash=$crash"
    echo "════════════════════════════════════════════"

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
