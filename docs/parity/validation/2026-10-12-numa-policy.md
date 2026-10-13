# NUMA exec policy validation — 2026-10-12

AI-generated implementation and review, GitHub #22 / RMU-34. Exact clean
implementation `8e6390df096fd310f371a6225650ffc7bbae225e`.

A fresh locked archive build of prior revision
`d2a7290ec3a7b715760f115addc8ffcf8dae9464` reproduces policy loss in both
direct and process-file exec, while init retains MPOL_LOCAL. The candidate and
pinned signed runc both retain it across all three forms and clean up. The
aggregate records include binary, suite, image and source provenance.

| Check | Result |
| --- | --- |
| Formatting / warnings-denied locked all-targets Clippy | Pass |
| Locked workspace tests | 24 pass |
| Architecture / offline inventory | Pass |
| Standalone label tests | 2 pass |
| NUMA kernel differential | Four matching groups: init, direct exec, process-file exec and cleanup |
| Existing exec differential | Four matching groups, two known differences retained |
| Vendor patch reproduction | Pass against checksum-pinned package |

One local test command misspelled the linker environment variable and failed to
find the development library. Corrected setup and the exact-revision test rerun
passed; no source/isolation change was needed.

```sh
python3 scripts/exec-differential.py --suite numa \
  --reference work/reference-runc/runc.amd64 --output work/numa-differential.json
python3 scripts/exec-differential.py \
  --reference work/reference-runc/runc.amd64 --output work/exec-differential.json
```

The focused suite adds only memoryPolicy to a private Podman-prepared OCI bundle
through a fixture adapter that forwards to the native runtime. It checks actual
procfs policy mappings, without publishing their contents or node IDs. Successful
init policy is checked independently from exec policy. No kernel policy or
security control is disabled. Binaries perform native runtime execution.

Review inspected tenant adaptation, pinned spec/reference init and setns behavior,
stored policy provenance, kernel reproduction, fixture boundaries, exact runner
provenance, cleanup and committed decisions. No unsafe/dependency/lockfile changes.
Other modes/flags, kernel denial, rootful, architectures and concurrency remain
open. Unsafe-sized node expansion and post-acquisition policy validation are
separate urgent #23 / RMU-35 gaps; do not claim complete NUMA support or parity.
