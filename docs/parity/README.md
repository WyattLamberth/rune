# runc compatibility baseline and evidence

Baseline established 2026-10-12 under GitHub [#11](https://github.com/WyattLamberth/rune/issues/11)
and [RMU-28](https://linear.app/rmux/issue/RMU-28).

| Source | Release | Exact revision |
| --- | --- | --- |
| [runc](https://github.com/opencontainers/runc/tree/29dd3dc2b13b4123162e5fe132504bb4b15569f1) | v1.5.2 | `29dd3dc2b13b4123162e5fe132504bb4b15569f1` |
| [OCI runtime-spec](https://github.com/opencontainers/runtime-spec/tree/92249139eea7161e13745abd4cb6d0ea02a3227a) | v1.3.0 | `92249139eea7161e13745abd4cb6d0ea02a3227a` |

The runc release's `go.mod` selects runtime-spec v1.3.0. These pins identify
source behavior, not an installed or validated reference binary. Build tooling,
feature selections, conformance tooling, fixture digests and reference binary
digests still need pins under #13. Neither upstream suite has been run against
rune. Changes to these baselines require an issue, a development-log decision,
regeneration, a reviewed inventory diff and fresh differential evidence.

## Source inventory

[inventory.json](inventory.json) contains a deterministic, issue-linked source
inventory, including all 17 registered commands, 106 option declarations
(including aliases and three framework help/version entries), 320 schema
properties, 328 Go JSON fields, 273 specification sections, 463 lines with
normative keywords, 32 runc documentation sections, 353 integration cases and
201 unit/fuzz entry points. Source SHA-256 digests allow regeneration to detect
changes. A schema pointer identifies a declaration, not a fully expanded
configuration path; `$ref` declarations are inventoried in their defining files.
The Go field inventory catches differences between the schema and typed model.

Resolve a `source` by its project prefix, the baseline repository and revision,
then its relative path. `L<number>` locators link to GitHub line anchors; schema
locators are JSON pointers; CLI locators identify commands/options; unit locators
identify Go functions. The inventory stores identifiers and links to source
rather than duplicating upstream prose or test bodies.

This is an inventory floor, **not an exhaustive semantic parity matrix**.
Normative lines can span multiple requirements, and keywords inside examples
are included conservatively. Markdown setext headings, generated test cases,
Go subtests, option value domains/defaults, annotations, schema constraints,
feature combinations and internal behavior require further refinement. Top-level
OCI platform branches are included even when runc does not implement them;
they must be resolved by evidence about the reference runtime's rejection
behavior, rather than omitted. Command options are extracted from the pinned
declaration layout; a baseline update requires re-auditing the extractor.

Statuses mean: **verified** has exact-revision semantic evidence; **partial**
has implementation and narrower evidence with obligations remaining; **missing**
has no implementation or explicitly rejects the feature; **untested** lacks
semantic evidence. No entry is currently verified. Existing alpha evidence is
reported in PR #10 at `8979560317d924c12bc5f21ee9ef2f252b44754b`; it is neither
a new run nor differential evidence. The eight implemented commands remain
partial. Options and fields remain untested even when parsing succeeds.

Each entry has a GitHub gap owner. #11 owns semantic inventory refinement;
#12 owns commands/options; #13 owns upstream tests, environment coverage and
the differential harness; #14 owns field semantics and fail-closed auditing.
These are refinement parents: split concrete discrepancies before implementation.
Future verified entries require an `evidence` item with `rune_revision` (full
commit SHA) and `record` (a versioned local evidence document). That record must
identify reference revision/build, case, configuration, expected and observed
behavior, validation layer and limitations. The integrity checker enforces
record presence; AI-generated review must assess its substance.

## Cross-cutting parity obligations

| Obligation | Current status | Gap owner and acceptance direction |
| --- | --- | --- |
| Lifecycle barrier, state, signals, exits | Partial | #6, #13: differential create/start/state/kill/delete and real create barrier |
| IDs, bundle paths, bounds, unknown/recognized fields | Partial | #2, #14: adversarial cases before acquisition, reject silently ignored semantics |
| Rootful and rootless, mappings/delegation | Untested for parity | #7, #13: equivalent kernel/configuration cases and explicit permission failures |
| Cgroup v1/v2, systemd/cgroupfs, every resource controller | Partial | #7, #13, #14: actual enforcement and update/event behavior |
| Namespaces, rootfs, mounts, devices, networking and time | Partial | #7, #13, #14: hostile path/mount cases and kernel membership/visibility |
| Capabilities, no-new-privileges, seccomp/notify, SELinux/AppArmor | Partial | #7, #13, #14: actual denial/inheritance, failure and cleanup |
| Intel RDT, scheduler, affinity, memory policy, personality | Untested or missing | #14: field audit, rejection or demonstrated semantics |
| Hooks, environment/cwd/user, rlimits, umask, stdio | Untested for parity | #7, #13: ordering, timeout, failure, child exit/status semantics |
| PTY, resize, attach, console/pidfd socket, inherited FDs | Partial | #12, #13: descriptor ownership, interactive behavior, leaks and interruption |
| CLI defaults/aliases, logging/errors, feature negotiation | Partial or missing | #12, #13: values, output/exit semantics, malformed input and feature claims |
| Checkpoint/restore and CRIU integration | Missing | #12, #13: native integration, version/features, rollback/recovery |
| Concurrent operations, stale PID/generation, crash recovery | Partial | #3, #4, #6, #13: kernel races plus deterministic production policy replay |
| Rollback, failed/interrupted deletion and cleanup retry | Untested for parity | #3, #4, #6, #13: acquisition fault matrix, no falsely successful cleanup |
| Linux architectures and kernel feature/configuration matrix | Untested for parity | #1, #13: cover reference support; unavailable environments stay gaps |
| Podman, containerd, Docker and relevant ecosystem integration | Partial | #7, #13: pinned integrations with disposable resources |
| Performance and reliability improvements | Unmeasured | #8: reproducible baseline and acceptance criteria after parity evidence |

The source inventory and these obligations must expand into exact semantic
cases. Policy simulation proves production policy; it does not prove kernel
isolation or concurrent syscall behavior. Complete parity requires evidence for
every applicable entry and configuration at the delivered revision, including
errors and rejected settings. Skipped or unavailable cases remain gaps.
If compatibility conflicts with the principles, document and investigate a
compliant design; leave the conflict unresolved in the matrix until proven.

## Reproduce and check

Offline integrity check (also in CI):

```sh
python3 scripts/parity-inventory.py
```

Clone the releases into ignored local work space without executing their tests:

```sh
git clone --depth 1 --branch v1.5.2 https://github.com/opencontainers/runc.git work/parity-upstream/runc
git clone --depth 1 --branch v1.3.0 https://github.com/opencontainers/runtime-spec.git work/parity-upstream/runtime-spec
python3 scripts/parity-inventory.py --runc work/parity-upstream/runc --spec work/parity-upstream/runtime-spec
```

Both repositories must be clean at the pinned commits. Source checking compares
the regenerated inventory and digests; offline checking only verifies structural
integrity. `--write` explicitly regenerates and resets statuses/evidence; preserve
reviewed semantic evidence separately before regeneration. Do not invoke upstream
integration helpers on shared host storage: fixed IDs and host assumptions need
isolation and an audited runner first (#13). No host configuration changes are
needed for this source inventory.

## First recognized-field evidence

The four memoryPolicy schema entries are partial, with exact evidence at
`ce207b9df26a2e8f3d8ff8bbed8eb738505bc425`. This covers rootless Linux amd64
MPOL_LOCAL init/direct/process-file retention, bounded node parsing, existing
mode/flag preflight rejection, and three safe kernel negative configurations.
It does not verify every mode, flag, syntax, platform or concurrency cross-product.
See `validation/2026-10-12-numa-bounds.md` and #22/#23/#14. No inventory entry
is fully verified. Source declarations still require exhaustive semantic cases.

## JSON input policy conflict

[#27](https://github.com/WyattLamberth/rune/issues/27) records nonregular JSON
rejection at `8c8c4e25d27971aa4769f80080b2d506c88d5599`. A nonblocking descriptor
open and regular-file check cover bundle, supplied process and stored exec reads,
preserving regular-file symlinks. Pinned runc accepts streamed FIFO input and can
block indefinitely. Rune explicitly rejects it under its finite-input contract.
This remains an unresolved compatibility/principle conflict, not verified parity.
[Exact evidence](validation/2026-10-13-config-input.md) distinguishes standalone
process/filesystem rejection from separate focused kernel regressions. #2/#14
retain the obligation to investigate a compliant streaming design and expand the
semantic input matrix. No source inventory entry is promoted to verified.
