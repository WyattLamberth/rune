# Continuous rune work checkpoint

Updated 2026-10-12; AI-generated engineering. Goal remains active.

## Current increment — cleanup metadata

GitHub #17 / RMU-33, branch `codex/17-cleanup-metadata`, stacked on differential
PR #19. Implementation `32a2c370cc695ce1a38d59a5cbd3bb5f45e7738f` requires
bounded regular cleanup metadata before delete effects and retains ownership
records on required cleanup failure. All local gates, 21 tests, general kernel
smoke and pinned exec differential pass; exact evidence is in
`docs/parity/validation/2026-10-12-cleanup-metadata.md`. The original intermittent
failure remains unresolved. Next: retain records on failed create rollback, then
improve harness cleanup errors and bounded failure diagnostics.

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

1. Publish/attach the cleanup PR after final-head checks and reconcile RMU-33.
2. Fix create rollback ownership loss after resource cleanup errors under #17.
   Then preserve smoke operation/cleanup failures together and bounded diagnostics.
3. Continue recognized OCI field semantic audit under #14/RMU-29 and expand
   signed differential/conformance coverage under #13/RMU-30. CLI #12 remains
   ordered behind security and validation evidence.

PR train: alpha #10, baseline #15, exec #18, differential #19. All prior hosted
Rust/Nix checks pass. Broader issues remain open; no complete parity claim.
Check current Git/PR/Linear state before repeating or changing completed work.
Ignored work holds private research inputs, reference assets, test diagnostics
and build artifacts. Preserve privacy and disposable resource boundaries.
