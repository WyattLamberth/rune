# Development log

AI-generated engineering and review under WyattLamberth's product direction.
Entries record evidence and uncertainty; no human review is implied.

## 2026-10-12 — Earlier alpha decisions reconstructed from recorded evidence

This entry is a **later reconstruction**, not a contemporaneous record of the
initial implementation. Sources: commits `a9ba97984ccc2c9fedb538e7a10c41070b5df91a`,
`11515c5558a76215fbf6bc4e70e335d08e3329cd`,
`8979560317d924c12bc5f21ee9ef2f252b44754b`, [PR #10](https://github.com/WyattLamberth/rune/pull/10),
GitHub #1–#9, architecture/security documents and vendor provenance.

The existing alpha uses libcontainer 0.7.0 as a native Rust library. Rune crates
forbid unsafe code and the policy core has no dependencies. The separately
disclosed backend has unsafe internals and local isolation/state patches.
Production dispatch is exercised by policy simulation; the full resource ledger,
generation protocol, crash recovery and kernel-port simulation remain open.
The backlog proposed a safe helper feasibility study (#5); the existing native
adapter provides a different bootstrap path. The retained records do not establish
a completed comparison of every alternative or #5's acceptance criteria.

PR #10 reports rootless Podman smoke and Rust/Nix validation at `8979560`.
Those are prior reported results, not results rerun for this entry. No runc
differential, complete OCI conformance, broad rootful/architecture evidence or
measured performance exists in the retained records. The support matrix already
distinguishes tested subsets from gaps. Preserve this boundary while expanding
the project mandate.

## 2026-10-12 — Pin source baseline before expanding compatibility (#11 / RMU-28)

Problem: the new continuous mandate requires demonstrable complete runc parity,
but the alpha matrix lists only broad areas, has no pinned reference runtime and
has no readable versioned decision log. CLI parsing and typed field acceptance
cannot establish observable behavior or isolation.

Evidence: inspected all three existing commits, current source/configuration,
PR #10, GitHub #1–#9, the rune Linear project/issues/milestones and upstream
registered commands, option declarations, specification schema/model, normative
documents and tests. runc v1.5.2's `go.mod` uses runtime-spec v1.3.0.

Decision: pin runc `29dd3dc2b13b4123162e5fe132504bb4b15569f1` and runtime-spec
`92249139eea7161e13745abd4cb6d0ea02a3227a`; generate a source inventory with
digests, stable locators, gap owners and honest support statuses. Keep broader
failure/concurrency/recovery/cleanup/platform/ecosystem obligations visible.
Use GitHub #12 for command/option refinement, #13 for differential/conformance
harness and environment coverage, and #14 for recognized-field semantic audit.
Prioritize isolation audit and a trustworthy harness before broadening the CLI.
Linear's new Verified runc parity milestone has evidence-based acceptance.

Alternatives: a hand-maintained broad matrix would repeat the existing omissions;
declaring upstream tests exhaustive would miss generated cases, annotations,
constraints, value domains and adversarial cross-products. Automatically marking
accepted fields supported would fabricate semantics. The chosen inventory is
reproducible and conservative, but intentionally remains a floor for semantic
refinement, rather than a completed exhaustive behavior model. The runtime is
unchanged by this increment. Baseline changes require an explicit decision and
fresh validation; engineering time and existing code do not constrain redesign.

Validation: see versioned records in `docs/parity/validation/` and the issue-linked
PR for exact tested revisions. Only successful recorded checks count. The local
environment currently has Rust 1.96.1, but no `go`, `runc` or `nix` executable;
no differential, Nix, rootful/rootless or conformance run is claimed here.
Source checkouts are research inputs, not runtime dependencies. Existing alpha
evidence remains historical and scoped to its tested revision.

Remaining uncertainty: expand normative lines into requirements; refine Go
subtests/dynamic cases, schema constraints/references, CLI defaults/value domains,
annotations and every configuration cross-product. Pin build/test tooling and
test both runtimes with audited disposable resources. Audit recognized fields
for silent omissions before relying on backend acceptance. Missing decision
records and incorrect Linear status remain review defects.
