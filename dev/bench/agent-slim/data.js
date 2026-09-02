window.BENCHMARK_DATA = {
  "lastUpdate": 1788389383448,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "agent-slim benchmarks": [
      {
        "commit": {
          "author": {
            "email": "danilochiarlone@gmail.com",
            "name": "Dan Chiarlone",
            "username": "danbugs"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65916668b7553f058722185033a7b100aabf301f",
          "message": "ci: add Linux CI workflow (#2)\n\n* ci: add Linux CI workflow\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* style: cargo fmt\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: mkdir .snapshots parent dir in bench recipe\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: remove kernel-verify from gate until binary is rebuilt\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* feat: dockerize kernel build for reproducibility\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* chore: rebuild kernel binary with Docker (ubuntu:24.04 gcc-13)\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: simplify kernel-verify (Docker handles deps), re-add to gate\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: add test_timeout to known_failures, fail on crash/error\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: conformance recipe exits non-zero on failures/crashes\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: build all runtimes, bench agent, readable summaries\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: update actions to node24, fix bench summary extraction\n\n- actions/checkout v4.2.2 -> v7.0.1 (node24)\n- Swatinem/rust-cache v2.7.8 -> v2.9.2 (node24)\n- fix bench summary: use awk to extract table between box footer and JSON line\n  (sed pattern used unicode box-drawing chars that didn't match ASCII output)\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* refactor: deduplicate scratch-size lookup in justfile\n\nExtract the 4 duplicated 12-line case blocks into a single\n_scratch-mb private recipe. Adding a new runtime now means\nupdating one place instead of four.\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: use just build-all-rootfs instead of manual loop\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: cache rootfs CPIOs across runs\n\nSkip Docker rootfs builds when Dockerfiles haven't changed.\nEach job caches its CPIOs keyed on driver file hashes:\n- test: all rootfs CPIOs (+ .NET SDK, Go, cpio skipped on hit)\n- conformance: python + conformance CPIOs\n- bench: per-runtime CPIO\n\nAlso removes redundant 'cargo build --release' step from bench\n(the just bench recipe already calls 'just build').\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n---------\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>",
          "timestamp": "2026-09-02T15:45:53-07:00",
          "tree_id": "bfb6d61e4b83ff8d9b1755c1a729e9ee1ff56041",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/65916668b7553f058722185033a7b100aabf301f"
        },
        "date": 1788389382487,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 533.998,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 535.125,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 548.225,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 14.2,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 8.404,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 29.382,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 12.594,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 6.492,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 28.79,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 0.914,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 0.825,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.102,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.335,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.137,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 11.347,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 20.334,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 11.039,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 43.183,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 19,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 17,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 19,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}