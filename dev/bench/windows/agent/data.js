window.BENCHMARK_DATA = {
  "lastUpdate": 1789161870008,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "agent benchmarks": [
      {
        "commit": {
          "author": {
            "email": "danilochiarlone@gmail.com",
            "name": "danbugs",
            "username": "danbugs"
          },
          "committer": {
            "email": "danilochiarlone@gmail.com",
            "name": "Dan Chiarlone",
            "username": "danbugs"
          },
          "distinct": true,
          "id": "f9d0fa22492afa5c1fe89b5e7c078e94fe3f337c",
          "message": "bench: report resident memory on Windows\n\nThe RSS line only existed on Linux (RssAnon from /proc), so the Windows\nbench summary lacked the density metric.  Windows now reports the\nworking set from GetProcessMemoryInfo, the counter family vmm-benchmarks\nuses there: guest memory is a section mapping, which the private-commit\ncounters do not attribute to the process.  Values are comparable within\nan OS, not across them.\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>",
          "timestamp": "2026-09-03T16:47:57-07:00",
          "tree_id": "41a3e81cd10063e2f8bcb277811c8986702a359f",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/f9d0fa22492afa5c1fe89b5e7c078e94fe3f337c"
        },
        "date": 1788480581153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 11002.436,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 11003.118,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 11111.217,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 71.704,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 53.88,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 130.146,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 18.266,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 8.649,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 33.44,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 41.971,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 41.589,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 42.221,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.228,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.217,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 8.853,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 27.759,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 12.758,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 57.4,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 876.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 876.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 876.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 23,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 20,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 32,
            "unit": "MB"
          }
        ]
      },
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
          "id": "707fce8f86b78bbbb183e3a46d35706290913460",
          "message": "Merge pull request #7 from danbugs/hyperlight-0.17\n\nMigrate to hyperlight 0.17 (crates.io) with driver dedup and release/CI tooling",
          "timestamp": "2026-09-10T13:47:22-07:00",
          "tree_id": "24e5029fcb022d636267b060fdda78da62c9295a",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/707fce8f86b78bbbb183e3a46d35706290913460"
        },
        "date": 1789075110247,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 15317.895,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 16234.979,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 16274.942,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 149.244,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 76.783,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 157.565,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 29.284,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 13.01,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 46.509,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 62.602,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 56.483,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 55.667,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.941,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.277,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 11.642,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 53.105,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 17.824,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 88.38,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 27,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 20,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 28,
            "unit": "MB"
          }
        ]
      },
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
          "id": "c8f50bf0d99c21de40d573fee1599506b2f2b48c",
          "message": "Merge pull request #8 from danbugs/repo-polish\n\nRepo polish: standard files, license headers, README, and python-shell rename",
          "timestamp": "2026-09-10T17:37:19-07:00",
          "tree_id": "5efb02e9c2735a8a3a5829b10eab4f38f703c4e9",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/c8f50bf0d99c21de40d573fee1599506b2f2b48c"
        },
        "date": 1789089167532,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 16169.748,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 16118.94,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 16208.759,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 144.623,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 78.229,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 192.973,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 28.955,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 13.707,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 48.78,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 55.883,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 55.638,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 56.104,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.87,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.358,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.828,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 44.61,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 19.787,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 74.531,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 20,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 33,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "danilochiarlone@gmail.com",
            "name": "danbugs",
            "username": "danbugs"
          },
          "committer": {
            "email": "danilochiarlone@gmail.com",
            "name": "danbugs",
            "username": "danbugs"
          },
          "distinct": true,
          "id": "5e5aa8c5b3e0da73c4d52377eb1bb195e54a46db",
          "message": "docs(fs): correct create_sandbox example to the 6-arg signature\n\nThe Rust example passed a bare `false` in place of the network policy and\nwas missing the listen-ports argument; create_sandbox takes\n(initrd, entry, scratch_mb, mounts, network, listen_ports).\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>",
          "timestamp": "2026-09-11T17:27:09Z",
          "tree_id": "50ed154778cd7231daaa0f8d5e0b0f8361ae1f18",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/5e5aa8c5b3e0da73c4d52377eb1bb195e54a46db"
        },
        "date": 1789148532518,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 11841.726,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 12050.487,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 12099.03,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 105.337,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 58.193,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 114.461,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 21.444,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 9.959,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 36.407,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 42.306,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 42.203,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 42.839,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.172,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.24,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 8.997,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 30.325,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 13.713,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 56.572,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.2,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.2,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.2,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 20,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 28,
            "unit": "MB"
          }
        ]
      },
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
          "id": "55d3e7b363dc3a6a42acc293487b042f608e99c4",
          "message": "Merge pull request #9 from danbugs/custom-kernel-and-tier2-tests\n\nCustom kernel support, .NET tier-2 tests, and Unikraft platform fixes",
          "timestamp": "2026-09-11T13:55:04-07:00",
          "tree_id": "ea0e3c5f41fae60982c33768bf20135bad4c4558",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/55d3e7b363dc3a6a42acc293487b042f608e99c4"
        },
        "date": 1789161867156,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 15757.461,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 16251.747,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 17032.255,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 164.894,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 83.943,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 199.249,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 32.964,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 13.616,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 48.62,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 65.805,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 55.612,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 58.41,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.88,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.364,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 12.926,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 64.717,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 19.691,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 98.3,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.8,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.8,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.8,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 20,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 32,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}