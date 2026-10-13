# Continuous rune work checkpoint

Updated 2026-10-12; AI-generated engineering. Goal remains active.

## Current increment — direct exec policy

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

1. Finish direct-exec final revision checks and AI-generated review, publish and
   attach its stacked PR. Baseline PR #15 is already published and hosted Rust/Nix
   checks passed at `d31afa97548e765ed1b1f46a5215d27c63c40487`; RMU-28 remains in
   review. Read PR/GitHub/Linear state rather than repeating completed work.
2. Begin GitHub #14 / RMU-29: audit recognized OCI fields against the pinned
   specification and vendored realization. Inspect relevant commits and the
   development log, split a concrete high-risk silent omission into an
   issue-linked increment, reject before acquisition and validate its semantics.
3. Begin #13 / RMU-30: use the signed, pinned reference artifact to establish an
   audited differential runner; source builds and conformance tooling remain
   additional obligations. Initially no Go/runc/Nix executables were installed;
   a local reference artifact is now available. Upstream tests use fixed IDs
   and host assumptions, so do not run them directly on shared host resources.
4. Refine CLI #12 only with the audit and harness evidence. Preserve explicit
   unsupported errors, privacy, unsafe-code forbid and dependency boundaries.

Tracked work was clean at the implementation revision validation. Ignored
`work/parity-upstream/` contains disposable research checkouts; `work/native-lib/`
contains the existing linker alias; build artifacts remain ignored. No user
files were modified outside the increment. Reconcile current working-tree state,
exact PR head, Actions results and Linear before continuing or pausing.
