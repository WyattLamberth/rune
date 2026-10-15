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

Exact implementation `8c8c4e25d27971aa4769f80080b2d506c88d5599` passes all 31 Rust
tests, formatting, warnings-denied Clippy, architecture/inventory and ten Python
regressions. Standalone signed-reference differential shows four matching rejection
outcomes and the documented FIFO policy difference. Rootless exec/NUMA and
failed-create rollback regressions pass with their previous scoped results.
Versioned evidence and the explicit parity conflict are recorded in
`docs/parity/validation/2026-10-13-config-input.md`; current continuity and matrix
notes retain the unresolved streaming obligation. AI-generated review confirms
descriptor verification, regular-symlink compatibility, state preservation and
prior decisions. Local Nix remains unavailable; verify hosted final-head checks.

## 2026-10-14 — Finish delivery by integrating the validated alpha (#30)

Problem: main a9ba97984ccc2c9fedb538e7a10c41070b5df91a still contained only
AGENTS.md, LICENSE and README.md. Eleven stacked PRs held the implementation;
publication and branch tests had been reported as progress without completing
main integration. The user identified the gap. This is a delivery workflow defect,
not evidence that branch implementation or validation had reached main.

Decision: combine the current hardened alpha through c614c12 with the sibling
cleanup-evidence documentation at 763740e, preserve both commit histories and
exact records, and land one reviewed/validated integration revision. Reconcile
continuity to reflect the delivery boundary and document known alpha limitations
in README. Validate the exact combined candidate before landing, then inspect and
validate main, reconcile the superseded stack and Linear, and record its revision.
Implementation and integration review are AI-generated; no human review implied.

Alternatives: merging only the original alpha would deliver older code lacking
already validated fixes. Continuing to add branches repeats the delivery gap.
Squashing the entire stack would discard coherent implementation history and
rationale; preserve ancestry and a merge that contains the cleanup record fork.
The fork changes only documentation; production code remains identical to the
validated nonregular-input head. No isolation or acceptance criterion is weakened.

Review: inspected command dispatch and ID/lock/storage boundaries, bounded
snapshot/process composition, production policy replay with seed/step/trace,
owned process/cleanup tests, backend provenance/local patch and mandatory gates.
The vendored maintained dependency's existing unsafe internals remain an explicit
trust boundary; first-party crates forbid unsafe. Complete field and race coverage
is still absent. The native run audit also found silent PID-file write errors;
#31 records source-only evidence and atomic-publication/owned-rollback acceptance.
Do not invent a kernel reproduction or hide that known experimental limitation.
All parity, cleanup, concurrency, streamed-input and platform gaps stay open.

Integration acceptance review found that the broad runner explicitly probed
exec seccomp/SELinux but lacked its own init seccomp mode/domain assertion.
Add init probes to the disposable run fixtures before calling the alpha acceptance
complete. Existing capability and no-new-privileges probes already exercise init.
The conditional SELinux domain assertion is not exhaustive MCS/denial evidence;
seccomp mode 2 proves filter installation, not all policy actions. Preserve those
limits in the final record. This changes validation only, not runtime semantics.

Combined candidate db3685c passes 31 Rust tests, ten Python fault tests, two label
tests, all static gates, input/exec/NUMA comparisons and failed-create rollback.
The checksum-pinned dependency patch exactly reproduces 73 Rust/TOML files.
Release candidate 300fc4c passes the complete disposable rootless alpha smoke,
including newly explicit init security and conditional SELinux results (both
init/exec domain checks ran and passed), plus confirmed cleanup. These are exact
scoped results, not complete conformance, denial, concurrency or recovery proof.
Versioned aggregate records and combined AI-generated review are in
`docs/parity/validation/2026-10-14-integration.md`. Final-head/hosted gates and the
actual main delivery revision must be checked and recorded before calling this
increment delivered. Earlier failing cleanup records and all broader gaps remain.

## 2026-10-14 — Reconcile completed smoke bounds before cleanup tracing (#17)

The separate active chat completed PR #26 at
`dd06f7dc5f5d33a448f2b9e622cb8499edba21a5` and is implementing #27 in its own
worktree. This chat continues #17 in the primary checkout. Merge the completed
4 MiB subprocess-output cap and recursive-state JSON handling into the cleanup
trace branch, retaining both histories and validation/decision records. Do not
modify the other active worktree or repeat its input-validation implementation.

Pinned Podman source's CreateExitCommandArgs preserves the runtime executable
but does not serialize per-invocation --runtime-flag arguments. Earlier fixtures
used that option for the private runtime root. This is a concrete candidate
explanation for sporadic retained state: the conmon cleanup command may target a
different root, and ContainerRun can discard its removal error while preserving
workload exit status. Verify actual requests before attributing the original
failure. The next fixture must guard private scope before invoking native runtime
commands, preserving PID/FD behavior and every isolation check.

## 2026-10-14 — Keep conmon callbacks in the private runtime scope (#17)

Evidence: source inspection at Podman 5.8.7
`c593b672bf3db1173aebea565ebf1a724ea196dc` finds CreateExitCommandArgs forwards
the runtime executable but not per-invocation runtime flags. Controlled request
traces in two development comparisons confirm delete calls omit the private
root argument for both rune and the signed pinned runc. Earlier --runtime-flag
fixtures therefore did not keep conmon cleanup in their intended state scope.
Podman can log removal errors and discard registry records, so that omission
provides a concrete mechanism for sporadic leftovers. The original unrecorded
rune failure still cannot be attributed retrospectively with certainty.

Decision: bind the fixture's explicit private state root in a generated runtime
launcher for every native invocation, including callbacks. Reject conflicting or
duplicate root arguments and expired/non-owned scopes. Use execv, preserving
PID, inherited descriptors and stdio; this is test fixture configuration, not
rune delegating execution to runc. Trace only operation/boolean scope categories,
under a 64 KiB bound with regular/no-follow descriptors; publish aggregate counts
without IDs, paths or workload contents. Keep launcher/trace artifacts privately
under ignored work so callbacks cannot fall into global default state roots.
Apply the binding to general, exec, NUMA and failed-create fixtures.

Alternatives: changing global defaults or abandoning private roots violates the
disposable-resource contract. Sleeping until leftovers disappear hides root
misrouting and does not repair callback configuration. Additional parent
processes/captured native stdio could alter runtime PID/descriptor behavior, so
use execv and confirm the production fixture preserves both with a real owned
process/descriptor regression. No runtime policy, unsafe, dependencies, locks or
isolation checks change. Scope requests remain kernel integration evidence, not
policy simulation or exhaustive concurrency validation.

Development: two comparisons pass the full general kernel suite and strict
cleanup for both runtimes. The second records two missing-root deletes for runc
and one for rune; each is bound to its private root before native execution.
Four fixture contract tests plus ten existing cleanup tests pass. Exact committed
validation follows; broader cleanup/crash/resource-failure acceptance remains open.

Exact implementation `26074b5492ed2f7ff28ce31c013bb36c8cad1623` passes 29 Rust
tests, 14 fixture/cleanup tests, static gates, label tests, all three signed
differential suites and failed-create cleanup. Both general runs record one
delete without its root argument, bind it correctly, and leave no runtime state.
The request trace does not capture individual command exit results; overall
workflow/state assertions remain the result evidence. Tighten test-child deadlines
and kill/reap cleanup for the FIFO and PID/descriptor fixture regressions before
final publication; this changes the test harness only.

## 2026-10-14 — Make fully absent forced deletion a production no-op (#34)

Evidence: reviewed core/adapter history from the alpha and exec snapshots,
pinned runc delete.go, #17 and a live rootless metadata-fault fixture. Missing,
malformed and FIFO cleanup metadata each reject forced deletion, preserve state
bytes and keep the owned init running. Restoring metadata permits deletion, but
subsequent frontend cleanup encountered an already-absent state directory and
rune returned an error. Pinned runc treats complete absence as forced success.

Decision: add internal Absent status/Noop effect to deterministic production
policy. No-op dispatch invokes no adapter and cannot signal/delete resources.
The Linux adapter observes directory absence under the validated-ID lock; only
ENOENT is absence. Other errors, partial/corrupt records, files and symlinked
state directories remain errors. Force absent succeeds without output; ordinary
absent deletion still fails. Update seeded policy replay with absence and retain
seed/step/bounded trace, explicitly distinct from kernel tests.

Alternatives: treating missing state.json as full absence discards unknown
resource ownership. Runc removes partial directories in that situation; rune
keeps them pending the acquisition ledger, an explicit compatibility difference.
Implementing the no-op only in an untested adapter branch would bypass production
policy/replay. A new effect makes absence semantics testable without adding OS
dependencies to the core. No unsafe/dependencies/lockfile changes.

Development: 30 Rust tests and warnings-denied Clippy pass, including deadline
protected absent/repeated force/ordinary/invalid-ID/partial/file/symlink cases.
Standalone signed-reference comparison matches complete absence and records the
partial-ownership difference. Live private kernel faults, restore/retry and
frontend cleanup now all pass. The earlier development cleanup failure is not
erased; its retained private resources were subsequently recovered.

Source audit also opened #35 for poststop warning/continuation/deletion-order
semantics. Pinned OCI requires warning and remaining-hook continuation, while
pinned runc's hook list returns at first failure and rune retains state before
hook completion. Kernel confirmation and a compliant compatibility design remain
open; this increment does not silently choose or claim both contracts.

## 2026-10-14 — Verify poststop contract and correct deletion timing (#35)

Evidence: reviewed pinned OCI lifecycle step 13/config poststop, runc
state_linux.go destroy and configs.Hooks.Run, rune hook execution/delete history
and CLI logging. A controlled live rootless two-hook probe makes the first hook
fail with exit 7, while the second would succeed. Both runtimes return delete
exit 1 and never invoke the second hook. Runc's first hook sees the state
directory absent; rune's sees it present, and rune retains state after the error.
All private resources are recovered after observation. These initial probes
are development evidence, not attributed to an exact committed fixture revision.

Decision: remove runtime state only after successful kernel/resource cleanup,
but before poststop execution, matching both pinned reference ordering and OCI
post-deletion timing. Keep the snapshot in memory for hook state/dispatch.
Metadata, signals, RDT, cgroup and directory-removal errors still return before
hook execution; no unknown ownership is discarded. This supersedes the earlier
#17 decision that retained state through hook completion. Once actual resources
are destroyed, poststop is a separate lifecycle stage with its own diagnostics.
Hook failures still propagate rather than disappearing, and reference first-hook
error behavior remains unchanged in this timing increment.

Unresolved conflict: OCI requires warning and remaining-hook/lifecycle
continuation, whereas pinned runc stops at the first error and returns failure.
The kernel probe confirms this is real, not just a source inference. Do not
claim OCI conformance or full hook parity. Investigate a compliant compatibility
contract separately; do not change default behavior silently or equate successful
resource destruction with successful hooks. Warning-format detection in the
probe is only a textual indicator; return/continuation results already disprove
conformance. Hook timeout and retry cross-products remain additional work.

Validation uses the same private-root binding and native PID/FD/stdio behavior.
Published observations contain only hook ordinal, boolean directory/state-field
presence and standard status; no IDs, paths or contents. No unsafe, dependencies,
lockfiles or isolation changes. The dependency patch reproduces the pinned
package. Exact committed validation follows.

Exact implementation `46effbc48f66c11a6b4e1e3a855e7a6786622285` passes the
committed two-hook kernel comparison: deletion ordering matches; the failure
exit, skipped second hook and OCI conflict remain explicit. Live metadata faults,
restore/retry and general comparison pass with no retained private resources.
30 Rust and 14 Python tests plus static gates pass. See the dated poststop
validation record. AI-generated review checked final history, superseded #17
rationale, patch reproduction and limited claims; timeout/continuation remains open.

## 2026-10-14 — Cover hook stdin delivery with deadlines (#38)

Evidence: production run_hooks regression on the #35 base supplies 256 KiB of
bounded annotations to a hook that never reads stdin and exits naturally after
four seconds. Despite its one-second timeout, the original adapter fails the
regression after four seconds. The wait deadline starts only after write_all.
Pinned runc configs.Command.Run starts its timer alongside managed stdin copying
and kills/waits the owned process on timeout. Independent source audit also
finds a detached wait thread and bare-PID timeout signaling in rune; no PID-reuse
attack has been claimed reproduced.

Decision: serialize before spawn, reject negative/unrepresentable timeout before
spawn, retain one Child owner, use safe nix fcntl on the existing stdin pipe for
nonblocking writes, and apply one deadline across delivery and exit waiting.
Close the writer after delivery so EOF consumers finish. On delivery/wait error,
kill then wait for the direct child; preserve the initiating error and any
kill/wait failures. BrokenPipe still permits successful early hooks, with actual
exit status authoritative. No unsafe/dependency/lockfile or isolation changes.

Alternatives: a detached stdin thread can remain blocked after deadline and
complicates cleanup; a socket changes observable hook stdin type. Keeping the
old wait thread can race reap with bare-PID signaling. Safe pipe polling keeps
ownership local and uses a bounded 5 ms sleep; no performance claim follows.
No timeout still permits an indefinitely running hook, as the contract does.
Spawn/system uninterruptible waits and descendants remain explicit gaps. This
change does not resolve OCI/reference poststop warning/continuation conflict.

Development reproduction fails on the unchanged base and passes after the fix.
The test invokes production dependency code and real processes, with a separate
bounded worker and finite fixture lifetime; it is not policy simulation. Exact
committed regressions, kernel comparisons and final evidence follow.

Development kernel comparisons match the deadline and EOF assertions. The
reference backpressure probe observes two hook processes and one still present
at immediate observation, alongside concurrent frontend cleanup callbacks;
rune observes one and it is absent after deletion. An initial fixture incorrectly
called this observation direct-child reaping and failed the comparison. Corrected
classification records a concurrency validation gap: no exclusive attribution
to the direct command or proven reference child leak. Both finite hook processes
terminate naturally within four seconds if not killed. Production adapter tests
separately verify the owned child is reaped. No concurrency-parity claim follows.

Exact implementation `5a7a6aa88acdacdf7cafc561e01a187b4b2544eb` passes
31 Rust tests (plus the explicitly invoked worker), 14 Python tests, static
gates and dependency patch reproduction. Signed-reference rootless deadline,
EOF, poststop failure ordering and general kernel comparisons pass their stated
assertions. Reference callback attribution remains a concurrency validation gap;
OCI continuation remains a conflict. See the dated hook-deadlines record.
AI-generated review inspected the production child lifecycle, error propagation,
no-unsafe boundary, regression limits, history and contemporaneous decision log.

## 2026-10-14 — Integrate cleanup and hook delivery with current main (#40)

Problem: reviewed PRs #33/#36/#37/#39 remain stacked on the already closed
pre-main train. Combine them with main f8fc760 without discarding descriptor
input validation, init-security assertions or either history. Publication is not
delivery; validate the combined revision before main integration.

Decision: merge main into an issue-linked integration branch, preserving both
ancestries. Resolve the development log by retaining both contemporaneous
records; update continuity to actual delivered main and current scope. Combine
fixture runtime-root trace and SELinux-domain result reporting while keeping
init/exec security assertions. No isolation assertion is removed. Input reader
and regression additions merge alongside absence/no-op policy and hook deadlines.
Review the merged implementation, fixtures, patch reproduction and decision
history; run all relevant exact Rust/static and signed-reference kernel suites.

Broader cleanup/concurrency, poststop OCI/reference continuation, bounded FIFO
streaming, platform and full-parity scope remains open. Separate PID-file work is
not imported or modified. AI-generated integration/review; no human review claim.

Hosted #39 Nix failed at the hook regression because the fixture hardcoded
/usr/bin/python3, absent in the build sandbox. Local and hosted Rust passed;
this was a test portability defect, not a passing Nix result. Replace the hook
fixture with an explicitly selected child mode of the same Rust test executable,
keeping every deadline/EOF/exit/negative-timeout/reaping assertion and the bounded
outer worker. This needs no interpreter or build/runtime dependency. Do not skip
checks or weaken the sandbox. The correction is part of combined #40 delivery;
#39's original failing checks remain recorded. Combined 937db3f passed all stated
kernel comparisons, metadata faults and input/absence checks before this fixture
correction. Failed-create invocation used an unsupported results flag and was
rejected before execution; rerun with its supported interface.

Combined `937db3f294e11ddd47f7f6754581f8fd871fc473` passes 33 Rust/14 Python/
two label tests and all mandatory static gates, plus seed-42 100000-step policy
replay. Input/absence, all three hook scenarios, exec/NUMA/preflight/general
signed-reference comparisons and live metadata fault/retry pass their scoped
assertions. Init/exec SELinux domain checks both run and pass. The corrected
sandbox fixture at 805b121 passes Rust/Clippy and failed-create rollback/private
cleanup. The full review and limitations are in the combined validation record.
Hosted combined checks and final-head general/hook gates remain required before
landing; publication alone is not delivery.


## 2026-10-14 — Independent project documentation (#43)

At the product owner’s request, remove unrelated project attribution and
workspace-specific delivery URLs from the README, architecture description and
parity baseline. Keep the engineering principles and GitHub baseline/backlog
links; describe Nix pinning through the committed lockfile. Remove those URLs
rather than invent replacement links. This documentation change leaves runtime
behavior and existing validation limitations unchanged. AI-generated review
checks the complete tracked-file search and documentation diff; formatting,
warnings-denied Clippy, architecture and workspace test gates validate the
proposed revision. Historical commits remain the engineering record.


## 2026-10-14 — CLI diagnostic pipeline (#45)

The CLI review identified backend events dropped without debug, warning-only
stderr in debug mode, mixed text/JSON output and two independently opened
writers. Pinned runc v1.5.2 `main.go:193–234` configures one logrus sink, enables
debug filtering and selects a JSON formatter. Its fatal writer routes terminal
errors to that logger and additionally to stderr when redirected.

Replace the split path with a small tracing layer and one synchronized,
synchronous writer. Emit level/message/RFC 3339 timestamps consistently, retain
extra backend event fields separately, and preserve the first write/flush error.
Opening the destination fails before runtime effects; write errors do not roll
back an already executed command, and cannot erase its nonzero status. Use a
typed format enum and borrowed root path. The default tracing JSON schema would
use different field names; a custom adapter keeps the expected level/msg/time
keys without adding a second serialization path. Clock and IO stay outside the
dependency-free core. Chrono/tracing already existed in the lockfile; direct
dependencies introduce no new packages.

The cgroup concern was checked against `init_builder.rs`, `state.rs`,
`tenant_builder.rs`, and lifecycle adapters: the backend persists use_systemd
and reloads it. This is source evidence, not new rootful/systemd kernel proof.
Missing commands remain #12; this increment makes no complete parity claim.
Regression tests cover level filtering, consistent schemas, append, backend and
final events, write/flush/open failures and process-local concurrent records.
The signed-reference comparison covers eight flag/destination combinations per
runtime. Cross-process ordering, caller/span metadata and byte-for-byte runc
text/error equality are not established. Runtime integration remains necessary
to check diagnostic initialization across backend process setup.

Product direction explicitly reaffirms that any part of rune, including the
entire implementation, may be replaced when evidence justifies it. Existing
code is no design constraint; isolation and exact-revision validation remain
mandatory. Implementation and review here are AI-generated.
