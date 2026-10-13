# Continuous rune work checkpoint

Updated 2026-10-13; AI-generated engineering. Goal remains active.

## Current increment — nonregular JSON input (#27 / RMU-36)

Branch codex/27-nonregular-config, stacked on PR #26. Exact implementation
8c8c4e25d27971aa4769f80080b2d506c88d5599 uses nonblocking open and verifies the
opened descriptor is regular before bounded JSON reads. Regular-file symlinks
remain supported. 31 Rust tests, static gates, ten Python regressions,
standalone input differential, focused exec/NUMA kernel differentials and
failed-create rollback pass. Evidence: docs/parity/validation/2026-10-13-config-input.md.
Runc streamed FIFO acceptance conflicts with rune's bounded-input requirement;
that unresolved conflict prevents complete parity. Final evidence/publication
checks follow; local Nix remains unavailable.

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

1. Finish final-head validation/publication for #27; retain the streamed-input
   compatibility conflict and investigate compliant bounded streaming behavior.
2. Reconcile the other active rune chat's #17 cleanup-ordering investigation before
   working on it; root cause, interrupted acquisition and concurrency remain open.
3. Review and integrate the validated PR train, preserving decisions and exact
   delivered-revision evidence. Do not infer integration from green branch checks.
4. Expand recognized-field audit #14 and exhaustive semantic/environment cases
   under #11/#13. Zero source inventory entries are fully verified.

The isolated branch used here is codex/27-nonregular-config. Inspect current Git,
GitHub and Linear rather than relying on this checkpoint's commit pointers as
final PR head. Private test resources and reference inputs stay ignored. All
private recovery resources created by this chat are cleaned.
