window.BENCHMARK_DATA = {
  "lastUpdate": 1789074178526,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "python benchmarks": [
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
        "date": 1788479971548,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 871.931,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 873.734,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1022.789,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 43.308,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 29.217,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 241.037,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 15.068,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 8.579,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 173.881,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.658,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.573,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 11.003,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.783,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.245,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 43.527,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 22.103,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 12.479,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 288.192,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 13,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 11,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 25,
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
        "date": 1789074176484,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 982.138,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 973.014,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1129.068,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 49.067,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 33.277,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 261.709,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 16.938,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 9.028,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 184.749,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.73,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.575,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 11.027,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.746,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.27,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 43.675,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 24.49,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 15.986,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 309.678,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 13,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 11,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 26,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}