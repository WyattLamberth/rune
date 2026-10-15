# Nonregular JSON input rejection — 2026-10-13

AI-generated implementation and review: GitHub #27 / Linear RMU-36. Exact clean
implementation `8c8c4e25d27971aa4769f80080b2d506c88d5599`, stacked on PR #26.

| Validation | Result |
| --- | --- |
| Formatting / warnings-denied locked all-targets Clippy | Pass |
| Locked workspace tests | 31 pass |
| Architecture / inventory integrity | Pass |
| Python cleanup control/filesystem regressions | 10 pass |
| Standalone signed-reference input differential | Four matching rejection outcomes; one documented FIFO policy difference |
| Rootless signed-reference exec differential | Four matching groups; two existing known differences |
| Rootless signed-reference NUMA preflight differential | Five matching groups |
| Rootless failed-create rollback | Missing/non-executable commands and cleanup pass |
| Local Nix | Executable unavailable; hosted checks required |

The new CLI test uses a two-second kill/reap deadline so a reintroduced blocking
open fails the regression rather than hanging the suite. Bundle, supplied-process
and stored exec FIFOs all fail with the regular-file diagnostic. Create acquires
no container records; exec preserves existing state bytes and an owned live
process. Direct reader tests reject a directory and a character device and
accept regular-file symlinks across all three paths. These are process/filesystem
validation tests, not kernel isolation or concurrent-command acceptance.

Adjacent aggregate records pin the rune and signed runc binary digests, source
revisions, Cargo.lock and suite/helper digests. The standalone input fixture uses
writerless FIFO, directory, device symlink, malformed regular JSON and its regular
symlink. Rune promptly rejects every fixture; pinned runc times out on the FIFO
and rejects the other four. Matching rejection outcomes do not prove identical
error text, every accepted file shape or exhaustive parity. The 500 ms observation
deadline is a test bound, not a performance measurement.

Compatibility conflict remains open: runc loadSpec in pinned spec.go uses
ordinary os.Open and allows streamed configuration; rune rejects nonregular JSON
under its finite-input contract. The byte bound previously failed to prevent
blocking open while holding a container lock. A safe bounded streaming design
that preserves the required compatibility remains unresolved under #27/#2/#14.
No complete parity claim is possible while that conflict remains.

AI-generated review inspected original reader history, cleanup metadata's safe
open precedent, pinned loadSpec, descriptor type verification before the bounded
read, regular symlink preservation, CLI ordering before acquisition, state/process
preservation and delivery context. No unsafe code, new dependency, lockfile change,
core OS access or weakened isolation. Arbitrary remote filesystem latency and
concurrent mutation of regular files remain outside this increment's proof.
