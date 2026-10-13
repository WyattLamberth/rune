# Cleanup metadata validation — 2026-10-12

AI-generated implementation and review for GitHub #17 / RMU-33. Exact clean
implementation revision: `32a2c370cc695ce1a38d59a5cbd3bb5f45e7738f`.

| Check | Result |
| --- | --- |
| Formatting / warnings-denied locked all-targets Clippy | Pass |
| Locked workspace tests | 21 pass |
| Architecture / offline parity integrity | Pass |
| Standalone label policy tests | 2 pass |
| Private rootless Podman general kernel suite | Pass, including state cleanup |
| Pinned-reference direct-exec differential slice | Four matching assertion groups; two known differences retained |
| Vendored patch reproduction | Applies to checksum-pinned package and reproduces all Rust/TOML files |

Reproduce with the commands in the exec-policy validation record, plus
`python3 scripts/exec-differential.py --reference work/reference-runc/runc.amd64
--output work/cleanup-differential.json` after preparing the authenticated reference.
Aggregate provenance is in `2026-10-12-cleanup-differential.json`. Both lockfiles
remain committed and unchanged. No unsafe code or dependency changes were added.

Review inspected delete/config source, original alpha history, bounds and
nonblocking regular-file checks, state-retention regressions, patched dependency
reproduction, development decisions and Linear acceptance. Metadata rejection
tests use stale identity fixtures and an owned process; they do not prove a live
container rollback or kernel isolation. General and differential successful
cleanup tests do exercise real rootless containers.

The original intermittent cleanup failure remains unexplained and unreproduced.
Create rollback can still remove ownership records after resource cleanup failure;
that is the next concrete defect. Atomic metadata acquisition, failure injection,
RDT, poststop retry and concurrent/interrupted recovery remain validation/design
gaps. This subset advances #17 without closing it or claiming parity.
