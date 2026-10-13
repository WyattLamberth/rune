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

## 2026-10-12 — Preserve stored process policy for exec (#16 / RMU-32)

Problem: direct exec reconstructed `ProcessBuilder` defaults in the backend and
lost stored AppArmor, user/group, cwd, rlimits, scheduler, I/O priority, OOM
adjustment and umask. Podman's process-file exec smoke did not cover that path.
Read the relevant alpha commit (`11515c5`), vendor provenance, current decisions,
GitHub #16 and Linear RMU-32 before changing it. Pinned runc `exec.go::getProcess`
uses configured process policy, and `container_linux.go::newInitConfig` supplies
container policy for omitted security/resource fields.

Decision: compose a complete owned process in rune's OCI adapter. Read the stored
create snapshot through the existing bounded, unknown-field-rejecting parser;
retain the immutable-bundle decision. Direct exec clones original policy and
applies explicit overrides. Process-file exec retains its own user/cwd/command
and inherits absent security/resource fields, including empty labels/rlimit lists.
Pass the resulting snapshot to the backend; no backend patch or new dependency.
The composition function has no filesystem or kernel effects and its production
path is tested directly. Linux/state access remains in adapters.

Alternatives: patching a few backend defaults would leave other accepted fields
silently dropped; cloning only named fields would repeat that risk. Reusing the
mutable bundle would reverse the earlier snapshot boundary. A full backend
replacement is unnecessary for this defect, but remains available when evidence
justifies it. No unsafe code, dependency or isolation relaxation is introduced.

Compatibility limits: process-file plus process overrides/command is explicitly
rejected because the backend otherwise ignored most such settings. Runc ignores
some combinations; this remains a documented compatibility gap for a compliant
design under #12/#16. Omitted process-file no-new-privileges inherits original
policy in rune; runc's boolean decoder can treat omission as false. Explicit
false remains distinct. Resolve this difference with differential/security
evidence rather than silently dropping requested isolation. Direct `--tty` and
explicit false CLI no-new-privileges remain unsupported.

Environment composition needed another correction: backend executable validation
reads the first PATH entry while environment installation uses last assignment.
Resolve duplicate environment keys to the last assignment before the backend,
preserving deterministic ordering. Tests cover repeated overrides and PATH,
retained complete process policy, capabilities without implicit defaults,
identity/group semantics, process-file fallback/explicit policy, malformed input,
conflicting options and bounded stored snapshots.

Development validation: 19 workspace tests and warnings-denied Clippy pass.
Disposable direct-runtime kernel checks pass UID/GID, cwd, rlimit, environment,
capabilities, no-new-privileges, process-file fallback, overrides, init identity
and cleanup. General Podman smoke passed on a diagnostic rerun; an earlier
uncommitted development run failed final runtime-state cleanup after its other
assertions passed. That failure was not captured with an exact source snapshot;
it remains GitHub #17 / RMU-33, not resolved by the later pass. No AppArmor denial,
full runc differential, complete cleanup or parity claim is made. Exact committed
revision evidence follows in the validation record and PR. Baseline PR #15's
hosted Rust/Nix checks passed at `d31afa97548e765ed1b1f46a5215d27c63c40487`.

## 2026-10-12 — Establish the first signed-reference differential slice (#13)

Evidence: read #13/RMU-30 and the exec design record, inspected the pinned runc
release assets, verified the amd64 checksum and detached signature against its
pinned source keyring, and checked reported revision/build provenance. The local
reference reports v1.5.2 / `29dd3dc`, Go 1.26.8, libseccomp 2.6.1 and libpathrs
0.2.6. It is an upstream release artifact; its build was not reproduced locally.
Exec PR #18's hosted Rust/Nix checks passed at `79bfe9951944bd4870dacb68d31dead1df09d787`.

Decision: start with a native direct-exec kernel slice using private disposable
Podman resources and the same pinned image/assertions for both binaries. Refuse
unknown reference hashes and dirty source for exact validation, build rune from
the clean revision, record binary/lock/suite digests, and invalidate claims if
source changes during the run. Save aggregate case classifications with explicit
known differences and untested gaps; never save host/workload logs in the record.
Reference preparation uses disposable GPG settings and does not install globally.

Alternatives: waiting for a full source-built toolchain would delay useful
differential evidence; a signed pinned release artifact establishes an initial
reference with explicit provenance limits. Running the entire upstream suite
unmodified would introduce fixed IDs and shared-host assumptions. Reporting all
successful expected outcomes as parity would hide intentional differences.

Development runs matched direct policy, overrides, init identity and cleanup
assertions. They also disproved the source-only expectation that runc would apply
the original file limit to an omitted process-file rlimit: actual exec retained
its parent's limit. Omitted no-new-privileges was false. Rune retains both
original policies. The runner now tests those observed differences explicitly;
this corrects the earlier incomplete source inference without fabricating prior
evidence. Ignored process-file overrides remain another known difference.
The initial failed development comparison is retained locally with validation
gaps; no exact revision is attributed to uncommitted development results.

Next obligations: commit and run the exact-revision slice, then expand security
denial/failure/concurrency cases and the source-built/conformance environment
matrix. Resolve policy/compatibility differences through compliant design and
evidence. No complete parity or exhaustive semantic coverage claim.

## 2026-10-12 — Preserve cleanup ownership on metadata failure (#17)

Evidence: reviewed the original alpha commit `11515c5558a76215fbf6bc4e70e335d08e3329cd`,
the exec decision record, #17/RMU-33 and the vendored delete/config paths. Delete
previously treated any cleanup-config load failure as permission to skip resource
cleanup and discard state. Intel RDT cleanup errors were also only warnings. This
is a concrete independent defect; it is not an established cause of the earlier
intermittent smoke failure. Differential PR #19 is published with passing hosted
Rust/Nix checks at `a78d051ef3acafca11f09ebc1ecc867ef3b7c858`.

Decision: require readable cleanup ownership before delete signals or persisted
status changes. Read at most 4 MiB from a regular, non-symlink file, use nonblocking
open to reject FIFOs without hanging, and reject unknown top-level fields. Return
metadata, RDT, cgroup and poststop errors instead of discarding ownership records.
Remove state only after required cleanup succeeds. No unsafe code or dependencies
were added; the existing maintained-dependency trust boundary still applies.

Alternatives: assuming missing metadata means no resources can leak existing
resources. Reconstructing ownership from a mutable bundle would violate the
stored-policy boundary and cannot prove what was acquired. A durable acquisition
ledger with atomic writes and idempotent replay is the stronger recovery design,
but requires a separate create/rollback increment; this change retains evidence
and fails explicitly until that design exists. Nested hook field validation is
not newly exhaustive. Interruptions before metadata creation can now require
explicit recovery; that gap remains visible rather than reporting success.

Development validation: 21 workspace tests passed, including missing, malformed,
unknown-field, oversized, symlink, directory and FIFO metadata rejection with
retained state and a live owned process. These are process/storage regressions,
not container-isolation tests. Warnings-denied Clippy, architecture/inventory and
the private general Podman kernel suite passed. The regenerated dependency patch
applies to the checksum-pinned package and exactly reproduces vendored Rust/TOML
files. Exact committed revision validation follows. Root-cause investigation of
the intermittent failure, interrupted acquisition, concurrent deletion, RDT
failure injection and retry/recovery acceptance remain open under #17/#13.

## 2026-10-12 — Retain acquisition evidence after failed create rollback (#17)

Evidence: reviewed the original alpha `builder_impl` history and cleanup records.
Its rollback collected cgroup/RDT errors but removed the state directory before
returning those errors, losing acquisition evidence needed for retry. PR #20
addresses delete-time metadata failure; it does not fix this separate create path.

Decision: continue attempting both resource teardown operations, preserve their
errors together, and remove ownership records only after both succeed. Extract
the actual filesystem finalizer into a safe backend module and exercise it from
workspace tests with resource-failure injection, retained bytes, successful retry,
already-absent state, removal failure and absent-container errors. A missing state
directory is idempotent success after resource cleanup; other removal errors are
returned. No unsafe/dependency/lockfile changes.

Alternatives: logging errors while deleting ownership loses recovery evidence.
Stopping at the first resource error unnecessarily skips the other teardown.
A full durable acquisition ledger remains necessary for crash recovery and
partial directory-removal failures; this increment does not claim that design.
Fault tests exercise the production filesystem finalizer, not injected kernel
cgroup/RDT failures or real concurrent rollback. Existing create outer/cleanup
error composition is preserved. Exact revision validation follows.

Development validation: all 24 workspace tests, warnings-denied Clippy and
architecture/inventory checks pass. The new private rootless create-failure
suite passes missing and non-executable commands, checking runtime state before
Podman removal can mask rollback failures. Finalizer fault tests retain records
after resource errors and remove them after a successful retry. Kernel resource
failure injection, leak enumeration, interrupted acquisition and concurrency
remain open; successful rollback tests do not close the intermittent #17 failure.

## 2026-10-12 — Preserve per-process NUMA policy during exec (#22)

Evidence: read tenant adaptation history, #14/RMU-29 and pinned OCI 1.3 Linux
memoryPolicy plus runc setns/standard-init code. The backend applies memory policy
to each init process, but its tenant Linux reconstruction dropped the field.
Namespace entry cannot inherit another process's NUMA policy. A private kernel
probe at prior head `d2a7290ec3a7b715760f115addc8ffcf8dae9464` found configured
MPOL_LOCAL in init but absent during rune exec; pinned runc retained it. A fresh
archive build of that exact prior commit reproduces direct and process-file
policy loss with the new focused suite. No giant-node inputs were executed.

Decision: copy the stored container memoryPolicy into the tenant spec alongside
other per-process container settings. Exercise native init, direct and process-file
exec by checking actual local-policy mappings in procfs, then verify cleanup.
A private fixture adapter adds only memoryPolicy to Podman's ordinary disposable
bundle before runtime create; all commands then invoke the native runtime. This
is test fixture preparation, not rune delegating execution to another runtime.
The signed-reference differential runner now accepts this focused suite and
retains its exact source/binary/lock/image/suite provenance.

Alternatives: relying on namespace membership silently drops policy. Rejecting
all NUMA policy would discard existing supported init behavior. Copying it
preserves the same native syscall path, while kernel differential evidence tests
its effect rather than only checking field presence. No unsafe/dependency/lockfile
changes or weakened isolation. Other modes/flags, rootful and architecture
cross-products, denied kernel features and concurrency remain gaps.

The same audit found unbounded node-range expansion and high-ID nodemask
allocation despite bounded JSON, with validation after acquisition. This separate
urgent defect is #23 / RMU-35; do not claim complete NUMA support until bounded
preflight and adversarial/failure evidence exist. Development MPOL_LOCAL init,
direct/process-file exec and cleanup pass for rune and the pinned runc.

## 2026-10-12 — Bound NUMA expansion and preflight policy (#23)

Evidence: inspected the native memory-policy parser/history and #23/RMU-35.
Small ranges could push billions of IDs; high single IDs could allocate enormous
nodemasks. JSON byte bounds alone did not bound derived work. Pinned runc uses
configs.ToCPUSet with a fixed unix.CPUSet-sized ID ceiling before conversion, so
its known behavior provides a stricter existing reference bound.

Decision: replace range-by-range expansion with a pure bounded node-set parser.
Reject IDs beyond the fixed CPU-set-sized ceiling, cap text at the existing 4 MiB configuration limit, sort and
merge intervals, then expand each unique node once. On the validated Linux amd64
reference the ceiling is 1023. This replaces the initially considered 65535
ceiling after upstream inspection found the tighter compatibility baseline.
The backend exposes its complete existing mode/node/flag validator for create
snapshot preflight and stored exec snapshot validation; actual native setup uses
the same parser and validation. No syscall or filesystem access enters the core.

Alternatives: checking only serialized size leaves range/mask expansion unbounded.
Counting each repeated range's expansion could reject small duplicate sets and
still do repeated work. Raising the ID bound beyond the pinned reference has no
parity justification. Merging intervals bounds duplicate work without weakening
requested policy. A considered 64 KiB text cap was replaced with the existing 4 MiB input
bound: interval merging already bounds expansion, and the smaller text cap would
add an unnecessary reference acceptance difference. Existing mode/flag rejection
remains unchanged; textual acceptance still needs its full reference matrix.

Tests exercise the actual production parser with malformed/overflow/giant IDs,
text bounds, repeated full ranges, overlapping sets and bounded exhaustive range
comparison. Adapter CLI tests check eight invalid policies produce errors without
container ownership records. Stored exec preflight rejects hostile policy and
accepts MPOL_LOCAL. An initial CLI fixture used the schema default development
version and failed at version validation; it now explicitly requests pinned
OCI 1.3 so assertions exercise NUMA rejection. Kernel differential and exact
committed validation follow. No unsafe/dependency/lockfile changes. Broader NUMA
modes, textual acceptance differences, denied kernels and concurrency remain gaps.

Development validation: 29 workspace tests and warnings-denied Clippy pass,
including CLI pre-acquisition rejection and stored exec checks. The rootless
preflight differential adds safe invalid MPOL_LOCAL nodes, absent MPOL_BIND
nodes and out-of-reference-range nodes, all failing with cleaned runtime state;
it then exercises valid init/direct/process-file policy and cleanup. All five
groups match in a dirty development run (no validated SHA). Giant ranges run only
through the newly bounded parser/adapter tests, never through the old runtime.
Patch reproduction passes against the checksum-pinned dependency archive.

## 2026-10-13 — Retain smoke cleanup evidence and preserve failures (#17)

Evidence: revisited smoke history and the missing original failure records. The
general suite ignored forced-removal results and could wipe owned state after
cleanup failed. Exec/NUMA/failure suites also removed their private trees even
when resources were not confirmed cleaned. The existing loop therefore could
lose the ownership evidence needed to diagnose intermittent leftover state.

Decision: use a shared bounded cleanup/diagnostic helper across all four suites.
Preserve operation and cleanup errors, enumerate only the generated private
Podman store, validate owned IDs, and reject uncertain cleanup. Inspect at most
32 state entries and 64 KiB per state record using anchored directory descriptors
and no-follow/nonblocking opens. Aggregate status/record categories exclude IDs,
paths, workload contents and logs. Retain uncertain resources with a mode-0600
private recovery locator under ignored work. Confirm private tree removal after
namespace cleanup; preserve subsequent removal errors. Add seven control and
filesystem fault tests to CI, including optimized-Python rejection and safe
timeout reporting. Differential provenance now includes the helper digest.

Development evidence changed the investigation: a broad signed-reference run
retained two runc records; another retained one runc record and one rune record.
Native state queries during recovery reported stopped for the latter pair; the
rune snapshot belonged to the controlled exit-status case. Both direct forced
deletes succeeded. Owned private trees were recovered, and no recovery locators
or test processes remain. Namespace tree cleanup once returned zero while a
residual tree remained; the helper now confirms removal and reports errors.
These runs had uncommitted harness changes and no exact candidate revision, so
they are development evidence only. They do not establish the original failure's
root cause, a runtime regression, concurrency correctness or comparative merit.

Alternatives: discarding records after errors hides failures; ignoring repeated
removal errors falsely claims cleanup. Waiting silently for state disappearance
would change the assertion without proving the expected asynchronous contract.
Retain the strict assertion and gather exact clean-revision evidence next. The
broader Podman comparison remains a validation gap until explained. No runtime
policy, unsafe code, dependencies, lockfiles or isolation settings changed.

Exact validation at `24387ac95edce4485816dd5cfd8e56b1c115e5a4` exposed a harness
contract defect: Podman ps defaults to shortened IDs, while the safe cleanup
validator requires full IDs. The helper rejected enumeration and retained the
owned containers; this is not runtime cleanup evidence. Request --no-trunc and
add an eighth fault/contract test. Recovery used full private IDs and native
delete for unregistered retained state, then confirmed removal of private trees;
stale locators whose trees were already absent were removed. No retained locator
remains. Preserve this failed exact report locally and validate the corrected
commit rather than treating the earlier run as a runtime regression.

## 2026-10-13 — Bound smoke subprocess output and tolerate hostile diagnostics (#17)

Evidence: reviewed cleanup helper introduction `24387ac95edce4485816dd5cfd8e56b1c115e5a4`,
#17/RMU-33, original smoke history and the retained aggregate development results.
The helper bounded subprocess duration but `capture_output` could retain unlimited
stdout/stderr. Its bounded state-file read could still raise RecursionError on
deep JSON, aborting diagnostic collection before the remaining cleanup attempts.
This is a harness robustness increment, not an explanation of runtime leftovers.

Decision: drain both subprocess streams with selectors and enforce a combined
4 MiB byte ceiling before retaining further data. Use one monotonic deadline for
pipe draining and process wait. Kill and reap the direct subprocess on failures;
the existing private-store cleanup remains responsible for owned container
resources. Preserve sanitized failure messages without command arguments or
captured output. Treat recursive JSON as unreadable state, retaining ownership
records through the existing cleanup path. No runtime, dependency, lockfile,
core-policy or isolation changes.

Alternatives: truncating output would let incomplete assertions appear valid.
Checking output size after communicate would still allocate unbounded memory.
Redirecting streams to files merely moves the unbounded resource to disk. Reject
an oversized response explicitly and retain the cleanup/recovery contract.
This runner does not claim to terminate arbitrary descendant process trees or
prove kernel resource reclamation. Rootful, concurrency and interrupted
acquisition acceptance remain open under #17/#13.

Development validation: nine Python control/filesystem regressions pass,
including an actual timed-out child, stderr overflow, both-stream exit handling
and deeply nested state JSON. Formatting and architecture/inventory checks pass.
The isolated worktree avoids concurrent edits by the other active rune chat.
Rust and kernel exact-revision validation follow after the implementation commit;
no dirty run is assigned an exact validated revision.

Validation and integration record: rebased the narrow output/JSON change onto
`7a609bc`, which independently corrects full-ID enumeration. Exact clean
implementation `aa2581ae1e6b65bfc94518a22f8658fecb7b58f4` passes ten Python
regressions, 29 Rust tests, formatting, warnings-denied Clippy, architecture and
inventory integrity, focused exec/NUMA differentials and failed-create rollback.
The broad differential retains one record in each runtime at cleanup-check;
subsequent native queries report stopped and direct deletion recovers both
private trees. Preserve this failure as a gap, not a claimed root cause. Local
Nix validation is unavailable because nix is not installed. AI-generated review
inspected history, production helper bounds, cleanup preservation and delivery
context. Exact aggregate evidence and continuation obligations are recorded in
`docs/parity/validation/2026-10-13-smoke-output-bounds.md`. The branch is isolated
from the other active rune chat; ongoing ownership selection is pending.

## 2026-10-13 — Reject nonregular OCI JSON before blocking reads (#27)

Evidence: inspected the original alpha File::open-based validation, subsequent
snapshot/NUMA commits, cleanup metadata's safe-open decision, #2/#14 and pinned
runc spec.go loadSpec. A private standalone writerless config.json FIFO fixture
at exact rune `dd06f7dc5f5d33a448f2b9e622cb8499edba21a5` and signed runc v1.5.2
hits its 500 ms deadline before parsing, without acquiring container records.
Rune holds the per-container command lock during that open. Supplied process
and stored exec configuration share the same vulnerable reader.

Decision: open nonblocking and inspect the descriptor's file type before reading
at most the existing 4 MiB plus sentinel. Reject nonregular JSON inputs explicitly.
Regular-file symlinks remain supported; descriptor inspection avoids a separate
path-metadata/open race. No unsafe, dependency, lockfile or deterministic-core
changes. This does not guarantee bounded latency for arbitrary remote filesystems
or change the snapshot's existing concurrent-write contract.

Compatibility conflict: pinned runc's ordinary os.Open accepts streamed FIFO
configuration and can wait indefinitely. Rejecting nonregular inputs is a known
policy difference, not runc parity. A compliant bounded streaming design remains
unresolved; a silent indefinite read violates rune's bounded-input requirement.
Do not mark this input contract verified for complete parity. Alternatives:
path metadata before open is racy; byte limits alone cannot bound open; supporting
unlimited writer waits preserves the defect. Rejecting all symlinks adds an
unnecessary compatibility difference, so inspect the opened regular file instead.

Production CLI regressions have a two-second kill/reap deadline and exercise
bundle, stored exec and supplied process FIFOs. Rejection creates no container
records, retains existing state bytes, and leaves an existing owned process alive.
A reader test rejects directories/devices and accepts regular-file symlinks in
all three paths. The standalone differential records artifact/source/fixture
hashes and the known FIFO timeout difference without logs or private paths.
Development: 31 Rust tests and warnings-denied Clippy pass. Exact committed
validation, isolated kernel regressions and a versioned compatibility record follow.
