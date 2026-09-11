window.BENCHMARK_DATA = {
  "lastUpdate": 1789147873947,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "python-shell benchmarks": [
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
        "date": 1788480051044,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1682.726,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1558.261,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1597.536,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 56.176,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 34.435,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 94.372,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 21.138,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 10.106,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 37.8,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 10.012,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.758,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 10.177,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 3.013,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.379,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.833,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 33.956,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 16.218,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 63.055,
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
            "value": 14,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 12,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 19,
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
        "date": 1789074243392,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1743.815,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1786.158,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1870.613,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 63.75,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 41.354,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 117.705,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 24.131,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 11.599,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 48.666,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 10.289,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 10.219,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 12.079,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 3.036,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.382,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 12.18,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 39.35,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 17.659,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 111.246,
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
            "value": 15,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 12,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 19,
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
        "date": 1789088025040,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1667.816,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1652.944,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1664.297,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 61.285,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 36.837,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 99.105,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 22.548,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 11.117,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 39.467,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.713,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.707,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 9.858,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.796,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.385,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.805,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 35.899,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 16.679,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 66.761,
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
            "value": 15,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 12,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 19,
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
        "date": 1789147871648,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1711.29,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1703.241,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1788.173,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 62.282,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 38.102,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 105.686,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 23.555,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 11.503,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 42.057,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.837,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.788,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 10.095,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.775,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.379,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.911,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 36.509,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 16.682,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 68.407,
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
            "value": 15,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 12,
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