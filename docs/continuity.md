# Continuous rune work checkpoint

Updated 2026-10-12; AI-generated engineering. Goal remains active.

## Current increment

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

1. Finish final revision checks and AI-generated review, publish/attach the
   stacked PR for #11 and keep RMU-28 in review until merged. Read PR/GitHub/Linear
   state on continuation to avoid repeating completed work.
2. Begin GitHub #14 / RMU-29: audit recognized OCI fields against the pinned
   specification and vendored realization. Inspect relevant commits and the
   development log, split a concrete high-risk silent omission into an
   issue-linked increment, reject before acquisition and validate its semantics.
3. Begin #13 / RMU-30: provision pinned reference build tooling and an audited
   differential runner. Initial environment lacks Go/runc/Nix executables;
   no reference/kernel/conformance run is claimed. Upstream tests use fixed IDs
   and host assumptions, so do not run them directly on shared host resources.
4. Refine CLI #12 only with the audit and harness evidence. Preserve explicit
   unsupported errors, privacy, unsafe-code forbid and dependency boundaries.

Tracked work was clean at the implementation revision validation. Ignored
`work/parity-upstream/` contains disposable research checkouts; `work/native-lib/`
contains the existing linker alias; build artifacts remain ignored. No user
files were modified outside the increment. Reconcile current working-tree state,
exact PR head, Actions results and Linear before continuing or pausing.
