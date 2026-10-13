# Continuous rune work checkpoint

Updated 2026-10-12; AI-generated engineering. Goal remains active.

## Current increment — NUMA exec policy

GitHub #22 / RMU-34, branch `codex/22-exec-numa-policy`, stacked on PR #21.
Implementation `8e6390df096fd310f371a6225650ffc7bbae225e` preserves stored
NUMA memoryPolicy during tenant composition. 24 tests, Rust gates, label tests,
NUMA and existing exec differentials pass at that clean revision. A fresh exact
prior-revision archive build reproduces both exec forms losing MPOL_LOCAL; the
candidate and pinned runc retain it. Exact evidence is in
`docs/parity/validation/2026-10-12-numa-policy.md`. Publish after final checks.
Next urgent increment: #23 / RMU-35 bounds node expansion and preflights policy.

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

1. Publish/attach NUMA exec PR after final checks; reconcile RMU-34.
2. Implement #23 / RMU-35 bounded NUMA node parsing and preflight before
   acquisition. Keep any explicit safety/compatibility limit visible.
3. Improve general smoke cleanup errors and aggregate failure diagnostics under
   #17, and expand interrupted/resource failure/concurrent cleanup evidence.
4. Continue recognized OCI field audit #14 and differential/conformance #13.

PR train: alpha #10, baseline #15, exec #18, differential #19, cleanup #20,
rollback #21. All prior hosted Rust/Nix checks pass. No complete parity claim.
Read current Git/PR/Linear state before repeating completed work. Private research
inputs, source archive builds, reference assets and diagnostics remain ignored.
