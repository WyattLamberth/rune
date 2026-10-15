# Continuous rune work checkpoint

Updated 2026-10-14; AI-generated engineering. Goal remains active.

## Current checkpoint — combine cleanup and hook delivery (#40 / RMU-42)

Primary checkout is on `codex/40-cleanup-integration`, combining main
`f8fc760029a051e80f7ae1f21884ca751e56d39d` with cleanup PRs #33/#36/#37/#39.
Preserve both histories and main's nonregular-input validation and init-security
assertions. Resolve documentation chronology and combine runtime-scope and
SELinux result fields. All exact combined gates, patch reproduction and kernel
suites follow before delivery; branch publication is not main integration.

PR #39 final `341751002725893fd2cada2843803f3a6a69d3c2` passes local Rust/static
and deadline/EOF comparisons. Implementation `5a7a6aa88acdacdf7cafc561e01a187b4b2544eb`
also passes all stated kernel regressions. Inspect hosted CI before integration.
PR #37 final `956a82997b672be871575c49e57f23c22c6b81be` passes hosted Rust/Nix.
OCI/reference poststop continuation conflict (#35) and concurrent callback hook
attribution (#17) remain open. No retained runtime records/recovery locators.

## Prior main integration — alpha and nonregular inputs

PR #32 merged the alpha/hardening; main is no longer documentation-only.
Main's exact evidence is `docs/parity/validation/2026-10-14-integration.md`.
#27 descriptor-based nonblocking regular-input validation and tests remain
required; streamed FIFO acceptance differs from runc and is an unresolved policy
conflict. Init seccomp and conditional SELinux domain assertions remain mandatory
in the broad runner; they are not exhaustive action/MCS/denial coverage.

Prior #34 / RMU-39 is published in PR #36 at
`ca9e25b357bd7fa4eb18808eb17d1a26c6190e99`; hosted Rust/Nix pass. Fully absent
forced deletion is a production no-op. Partial/corrupt ownership errors remain.

## Prior increment — callback runtime-root binding

PR #33, implementation `26074b5492ed2f7ff28ce31c013bb36c8cad1623`, final
`6cbcc7aafb82e2f0d44e4f3d33039e8f4537104a`; hosted Rust/Nix pass. Both general
cases observe omitted-root deletes, bind scope and pass strict cleanup. This
fixes fixture routing, not complete kernel recovery/concurrency.

## Prior increment — cleanup diagnostics and output bounds

PR #28 at `763740e8fd6bf024781af9c697d7f6b7591a1f28` passes hosted Rust/Nix and
focused kernel checks; its earlier general comparison retains a runc record.
Separate native recovery succeeds. PR #26 at
`dd06f7dc5f5d33a448f2b9e622cb8499edba21a5` adds output caps/recursive JSON
handling and passes hosted/local checks; broad earlier failures remain recorded.
The histories are reconciled in this branch.

## Parallel existing work

The other active “Read Codex goal objective” chat integrated alpha/hardening
via merged PR #32 and is handling separate #31 PID-file publication/rollback.
Do not modify its worktree. Reconcile main and open #33/#36 before integration;
do not send cross-chat messages without human authorization.

## Prior increment — bounded NUMA preflight

PR #25 / #23 / RMU-35, implementation
`ce207b9df26a2e8f3d8ff8bbed8eb738505bc425`, final
`3d3e377a7b89033b944a453eca6ae4780db4fe71`. Local and hosted Rust/Nix pass.
29 tests and five focused kernel differential groups pass. Four memoryPolicy
entries link partial evidence; none is fully verified.

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
GPG home; system trust settings were not modified. The signed-reference runner and narrow evidence are committed under #13;
source-build reproduction and full conformance remain open.

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
limits. It is a floor for an exhaustive semantic parity matrix. At that baseline, runtime
source remained the alpha. GitHub #12–#14 and Linear RMU-29–RMU-31 own CLI gaps,
validation tooling and isolation field audit. The Verified runc parity milestone
is open. Existing issues #1–#9 remain open because their broader acceptance
criteria are not completed by the alpha.

## Next work

1. Finish exact combined #40 validation, AI-generated history/code/decision
   review, hosted CI, main integration and authoritative delivery reconciliation.
2. Continue #35 OCI/reference continuation conflict and #17 concurrent delete /
   callback attribution, resource failures and interrupted acquisition.
3. Keep the other chat's separate #31 PID-file work isolated; inspect main before
   integrating and do not send cross-chat messages without human authorization.

Full parity remains unproven and goal active. Inspect current Git/PR/Linear and
private recovery locators; no retained test resource remains at this validation.
