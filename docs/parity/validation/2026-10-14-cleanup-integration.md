# Combined cleanup and hook delivery

AI-generated integration and review for #40 / RMU-42. Candidate
`937db3f294e11ddd47f7f6754581f8fd871fc473` combines main
`f8fc760029a051e80f7ae1f21884ca751e56d39d` with reviewed #33/#36/#37/#39,
preserving both Git histories and decision records. Production behavior adds
fully absent forced deletion, correct poststop deletion timing and safe hook
stdin deadlines/reaping. Fixtures keep callback root scope and all main init
security assertions. Nonregular input validation from main remains intact.

Exact candidate validation passes 33 Rust tests, 14 Python tests, two label tests,
formatting, warnings-denied Clippy, architecture/inventory checks and seed 42 /
100000-step production policy replay. The policy replay is not kernel isolation
or concurrency evidence. The checksum-pinned vendored patch reproduces all
Rust/TOML source differences. Cargo.lock and flake.lock are unchanged/committed.

Signed-reference comparisons pass all stated input, absence, hook backpressure,
EOF, failed-hook ordering, exec, NUMA, NUMA preflight and general lifecycle
assertions. Live cleanup metadata missing/malformed/FIFO faults preserve the
owned init and state; restore/retry succeeds. General cases retain explicit init
seccomp and init/exec conditional SELinux domain checks; both domain checks run
and pass for each runtime. This is not complete denial/MCS/action coverage.
Primitive aggregate evidence, exact binary/fixture/lock digests and known
classification limits are in the accompanying JSON reports. No private paths,
IDs, workload contents or logs are published.

Hosted #39 Nix fails because the Rust regression fixture hardcodes a Python
interpreter outside the sandbox. Correction `805b121` uses owned child/worker
modes of the same Rust test binary, retaining every assertion and deadline with
no new dependency or sandbox relaxation. 33 Rust tests pass with two explicitly
invoked ignored fixture modes. This changes tests and limitation documentation,
not production code. Failed-create missing/non-executable rollback and private
cleanup pass at 805b121 using the supported harness interface. The earlier
unsupported results argument was rejected before execution. Hosted combined
Rust/Nix and final-head kernel gates must pass before main delivery.

Unresolved differences remain: process-file exec policy/override behavior,
nonregular streaming input, partial ownership deletion, poststop warning and
continuation conflict, concurrent callback attribution, rootful/platform and
complete conformance/recovery. Reference backpressure observes two hook
processes alongside cleanup callbacks; direct ownership is not proven by the
shared marker. All finite hook fixtures end within four seconds if not killed.
Observed private records/registered resources are cleaned and recovery locators
are empty. Full parity remains unproven; none of those parent scopes is closed.

AI-generated review inspected final history, preserved prior failures, the #17
hook-order decision reversal, core no-op dispatch, ID validation/lock sequencing,
metadata-before-effects, resource errors before hooks, safe pipe delivery and
single Child ownership, input bounds, init security assertions and patch
provenance. Separate #31 work was not imported. No unsafe in rune crates,
dependency change, isolation weakening or unmeasured performance claim.
