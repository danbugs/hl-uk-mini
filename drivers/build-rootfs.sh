#!/bin/bash
# Build a rootfs CPIO from a driver Dockerfile.
#
# Usage: ./drivers/build-rootfs.sh python    → build-elfloader/python-rootfs.cpio
#        ./drivers/build-rootfs.sh node      → build-elfloader/node-rootfs.cpio
set -euo pipefail

DRIVER="${1:?usage: $0 <driver>}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$REPO_ROOT/build-elfloader"
DOCKERFILE="$REPO_ROOT/drivers/$DRIVER/Dockerfile"
IMAGE="hl-${DRIVER}-rootfs"
OUTPUT="$OUT_DIR/${DRIVER}-rootfs.cpio"

if [ ! -f "$DOCKERFILE" ]; then
    echo "error: $DOCKERFILE not found" >&2
    exit 1
fi

mkdir -p "$OUT_DIR"

echo "==> Building image $IMAGE from $DOCKERFILE"
docker build -t "$IMAGE" -f "$DOCKERFILE" "$REPO_ROOT/drivers/"

echo "==> Exporting to $OUTPUT (newc CPIO)"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

cid=$(docker create --entrypoint=/ "$IMAGE" 2>/dev/null || docker create "$IMAGE")
docker export "$cid" | tar -C "$tmpdir" -xf -
docker rm "$cid" > /dev/null

(cd "$tmpdir" && find . | cpio -o -H newc --quiet > "$OUTPUT")

echo "==> Done: $OUTPUT ($(du -h "$OUTPUT" | cut -f1))"
