window.BENCHMARK_DATA = {
  "lastUpdate": 1788479973715,
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
      }
    ]
  }
}