# Continuous rune work checkpoint

Updated 2026-10-14; AI-generated engineering. Work resumed by the user.

## Current increment — integrate the alpha into main (#30 / RMU-37)

Main at a9ba97984ccc2c9fedb538e7a10c41070b5df91a contains only documentation.
Eleven published PRs were mistakenly treated as the delivery boundary. The user
identified this gap; adding features is now secondary to finishing integration.
Branch codex/30-integrate-alpha combines the full validated stack through
c614c12ee338271d14c86447a7b4fd1c09b94a35 and cleanup-evidence documentation
763740e8fd6bf024781af9c697d7f6b7591a1f28. Runtime code is unchanged by that merge.
Preserve both histories and all exact evidence. Review the combined candidate,
run exact local/static/kernel gates, wait for hosted Rust/Nix, then land and
record the actual main revision. Reconcile the superseded PR stack and Linear.
Do not call publication delivery or imply human review.

Known alpha gaps remain: #17 cleanup/concurrency/recovery; #31 PID-file failure
handling (source-only finding); #27 streamed-input policy conflict; #11–#14 full
semantic inventory, CLI/field support and conformance/environment coverage.
Complete parity is unproven; integration must not close those parent issues.

## Prior increment — nonregular JSON input (#27 / RMU-36)

Branch codex/27-nonregular-config, stacked on PR #26. Exact implementation
8c8c4e25d27971aa4769f80080b2d506c88d5599 uses nonblocking open and verifies the
opened descriptor is regular before bounded JSON reads. Regular-file symlinks
remain supported. 31 Rust tests, static gates, ten Python regressions,
standalone input differential, focused exec/NUMA kernel differentials and
failed-create rollback pass. Evidence: docs/parity/validation/2026-10-13-config-input.md.
Runc streamed FIFO acceptance conflicts with rune's bounded-input requirement;
that unresolved conflict prevents complete parity. PR #29 final head c614c12ee338271d14c86447a7b4fd1c09b94a35 passes hosted
Rust/Nix and exact local/input/exec/NUMA checks. Local Nix is unavailable.

## Prior increment — smoke output bounds (#17 / RMU-33)

PR #26 final head dd06f7dc5f5d33a448f2b9e622cb8499edba21a5 passes hosted Rust/Nix,
29 Rust tests, ten Python regressions, static gates and focused exec/NUMA checks.
The broad differential retained one record in each runtime; later native queries
reported stopped and explicit native deletion cleaned both private trees. Exact
records remain committed. PR is ready for AI-generated review; original #17
root cause and concurrency remain open. The other active rune chat continues
broader cleanup investigation in its own checkout.

## Prior increment — bounded NUMA preflight

GitHub #23 / RMU-35, branch `codex/23-numa-bounds`, stacked on PR #24.
Implementation `ce207b9df26a2e8f3d8ff8bbed8eb738505bc425` bounds node IDs/text,
merges intervals before expansion, and validates create/stored exec policy before
acquisition. 29 tests, all Rust gates, label tests, NUMA preflight differential,
failed-create and general Podman suites pass. Five focused groups match pinned
runc. Exact evidence: `docs/parity/validation/2026-10-12-numa-bounds.md`.
Four memoryPolicy source entries now link partial evidence; zero entries fully
verified. Finish final checks and publication, then repair general-smoke cleanup
error preservation/diagnostics under #17 and review safe train integration.

## Prior increment — NUMA exec policy

PR #24 / #22 / RMU-34, implementation
`8e6390df096fd310f371a6225650ffc7bbae225e`, final head
`4418d360ff6e97283c58e98bbcb6bb7526be78b3`. Local Rust/kernel differential and
hosted Rust/Nix pass. Exact prior archive build reproduces dropped memory policy;
candidate and signed runc retain it. Broader NUMA configuration gaps remain open.

## Prior increment — rollback ownership

PR #21 / GitHub #17 / RMU-33, implementation
`6b226888ed1b119ae4b0a2daba16fff62c65c435`, final head
`d2a7290ec3a7b715760f115addc8ffcf8dae9464`. All local checks and hosted Rust/Nix
pass. Production finalizer retains records after resource errors and supports
retry. Actual kernel resource failure, interrupted acquisition, concurrency and
original intermittent root cause remain open.

## Prior increment — cleanup metadata

PR #20, implementation `32a2c370cc695ce1a38d59a5cbd3bb5f45e7738f`, final head
`34c118ab7b2f5d88b26160ae905337d66c10c4a1`, requires bounded regular metadata
before delete effects and retains state on required teardown failure. Local gates
and kernel suites pass; hosted Rust/Nix pass.
Original intermittent cleanup failure remains unresolved under #17.

## Prior increment — differential validation

PR #19 / GitHub #13 / RMU-30 is published at
`a78d051ef3acafca11f09ebc1ecc867ef3b7c858`; hosted Rust/Nix checks pass.
Implementation `e51c0f46d6a9a1a13fb2a41ddc9011844a2300b1` authenticates the pinned
signed upstream reference and runs a private rootless exec differential slice.
Four groups match; two are known differences, not parity. Source-build
reproduction and complete OCI/kernel/concurrency/recovery coverage remain open.

## Prior increment — direct exec policy

GitHub #16 / RMU-32, branch `codex/16-exec-process-policy`, stacked on parity
baseline PR #15. Implementation `3b6b8ab4c936d9cba29961da73de308b87ef75c4`
composes complete exec process policy in the OCI adapter from the bounded stored
create snapshot. Formatting, Clippy, architecture/inventory, 19 workspace tests,
2 label-policy tests, direct-exec kernel suite and diagnostic general Podman
smoke pass at that implementation revision. Evidence is in
`docs/parity/validation/2026-10-12-exec-policy.md`; final head evidence belongs on
the issue-linked PR. AppArmor denial and pinned-runc differential acceptance
remain open; do not close #16 from subset tests.

One earlier uncommitted general-smoke run failed final state cleanup and was not
captured with exact source/state diagnostics. Later passes do not resolve it;
GitHub #17 / RMU-33 owns the cleanup investigation and backend config-load audit.

Signed upstream runc v1.5.2 amd64 reference assets have been downloaded into
ignored `work/reference-runc/`. SHA-256 matches the release checksum, and detached
signature verifies against the keyring in the pinned source using a private local
GPG home; system trust settings were not modified. These research inputs still
need durable runner/build provenance and differential case evidence under #13.

## Prior increment — baseline

GitHub #11 / Linear RMU-28, branch `codex/11-runc-parity-baseline`, stacked on
alpha branch `codex/1-podman-alpha` / PR #10. Implementation commit
`f7019ce654f320b9ede04db3a28033ca99797249` establishes source pins, inventory,
decision log and CI integrity check. Exact tested implementation evidence lives
in `docs/parity/validation/2026-10-12-baseline.md`; final head evidence is recorded
in the issue-linked PR. Documentation evidence/checkpoint updates follow that
implementation commit. Inspect Git for the current head rather than treating
the implementation SHA as the final delivered revision.

The inventory has 2,093 entries, zero verified semantic entries, and explicit
limits. It is a floor for an exhaustive semantic parity matrix. All runtime
source remains the alpha. GitHub #12–#14 and Linear RMU-29–RMU-31 own CLI gaps,
validation tooling and isolation field audit. The Verified runc parity milestone
is open. Existing issues #1–#9 remain open because their broader acceptance
criteria are not completed by the alpha.

## Next work

1. Inspect main and GitHub #30/integration PR for the actual delivery revision
   and final checks. Finish only unmet integration gates; do not repeat a landed
   integration. Reconcile the superseded stack and Linear to authoritative state.
2. Resume #31 PID-file atomic publication/failure rollback, then native run
   parity; preserve cleanup failures rather than falsely reporting success.
3. Reconcile the other active rune chat's #17 tracing before touching that work.
4. Expand #11–#14 exhaustive parity cases, compliant streaming-input design and
   environment coverage. Zero inventory entries are fully verified.

Inspect live Git/GitHub/Linear state before resuming; implementation SHA pointers
are not a substitute for delivered main evidence. Private test resources and
reference inputs remain ignored. Never reset shared Podman storage or defaults.
