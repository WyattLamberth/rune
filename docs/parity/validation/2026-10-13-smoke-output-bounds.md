# Smoke output bounds and hostile state diagnostics — 2026-10-13

AI-generated implementation and review, GitHub #17 / Linear RMU-33. Clean
implementation `aa2581ae1e6b65bfc94518a22f8658fecb7b58f4`, stacked on cleanup
helper/full-ID correction `7a609bc`. Rune runtime source is unchanged by this
increment; the production smoke subprocess and diagnostic paths changed.

| Validation | Result |
| --- | --- |
| Python control/filesystem regressions | 10 pass |
| Formatting / warnings-denied locked all-targets Clippy | Pass |
| Locked workspace tests | 29 pass |
| Architecture / inventory integrity | Pass |
| Rootless exec differential | Four matching groups, two existing known differences |
| Rootless NUMA preflight differential | Five matching groups |
| Rootless failed-create rollback | Missing/non-executable cases and cleanup pass |
| Broad rootless Podman differential | Validation gap in both runtimes at cleanup-check |
| Local Nix check | Unavailable: executable not installed |

The pinned signed runc v1.5.2 artifact, upstream revision, binary and suite
hashes, Cargo.lock, toolchain and image digest appear in the adjacent exec,
NUMA and general aggregate JSON records. The reference has not been rebuilt
from source. These are rootless Linux amd64 subsets; no complete parity claim.

The broad run retained one state record for runc and one for rune after Podman
container enumeration was empty. Rune's persisted snapshot was running; a
later native state query reported stopped for both runtimes. Direct native
forced deletion succeeded for each, followed by confirmed private tree removal.
The aggregate recovery record preserves those results. No identifiers, host
paths, workload contents or logs are published. No private recovery resources
from this chat remain. This does not explain the original intermittent failure,
prove asynchronous ordering, or establish runtime comparative merit. A later
pass cannot close #17. Recovery is an explicit operation after evidence capture;
the suite retains its strict cleanup assertion.

Review inspected subprocess byte accounting and one deadline across both pipes
and process wait, direct-child kill/reap on errors, output decoding/sanitization,
recursive state rejection, cleanup error preservation and prior decision records.
No unsafe, dependency, lockfile, core-policy or isolation changes. The Python
fault tests validate harness control/filesystem behavior, not kernel isolation.
The differential and rollback runs provide separate real-kernel subset evidence.

Continuation: this isolated issue-linked branch adds output bounds and recursive
JSON handling to the separately maintained cleanup-evidence branch. Keep the
other active rune chat's broader #17 investigation independent. Verify final
PR head and hosted CI before integration; Nix remains an explicit local gap.
Next #17 work must investigate cleanup ordering, interrupted acquisition,
resource teardown failures and concurrency without silently waiting away failures.
