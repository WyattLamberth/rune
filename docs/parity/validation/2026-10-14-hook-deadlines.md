# Hook delivery deadlines — bounded evidence

AI-generated engineering and review for #38 / RMU-41. Implementation:
`5a7a6aa88acdacdf7cafc561e01a187b4b2544eb`. Pinned signed runc v1.5.2:
`29dd3dc2b13b4123162e5fe132504bb4b15569f1`; OCI runtime-spec v1.3.0:
`92249139eea7161e13745abd4cb6d0ea02a3227a`. Binary, lockfile and fixture hashes
are in the accompanying reports. The reference source build is not reproduced.

A development production-adapter regression supplied 256 KiB of annotations to
a nonreading hook with a one-second timeout and finite four-second lifetime.
The old adapter failed after four seconds because it wrote state before starting
the timer. This was development reproduction, not an exact committed-baseline
comparison. The revised production adapter uses one deadline across nonblocking
pipe delivery and exit waiting, closes stdin and retains Child ownership until
kill/reap. It rejects negative/unrepresentable timeout before spawning and
preserves initiating and child-cleanup errors. No unsafe/dependency/lock changes.

Exact committed validation passes formatting, warnings-denied Clippy,
architecture/inventory integrity, 31 Rust tests (one additional ignored worker is
explicitly invoked by the parent), 14 Python tests and dependency patch
reproduction. The real-process adapter regression covers backpressure timeout,
wait timeout, EOF, early success, nonzero exit and negative timeout; successful
and timed-out child PIDs are absent after return. Tests have bounded outer
workers. This is real kernel process/pipe coverage, not deterministic policy
simulation or complete container isolation testing.

Private rootless comparisons against the signed reference match:

- Large-state nonreading poststop reaches timeout and delete exit 1 within the
  three-second assertion bound; this is a deadline assertion, not a speed claim.
- EOF-consuming hooks finish, both hooks run, and delete succeeds.
- Failing first-hook deletion ordering remains matched, with OCI/reference
  warning/continuation conflict still explicit.
- General rootless kernel lifecycle/security assertions pass for both runtimes.

The reference backpressure probe observes two hook processes alongside a
frontend cleanup callback; one is present immediately after the direct command.
Rune observes one and it is absent. The report calls this a concurrency
validation gap: process markers are shared by callback invocations, so they do
not establish exclusive direct-child ownership or a leaked reference child.
All hook fixtures have finite four-second maximum lifetimes. Private runtime
records and registered test resources are removed and no recovery locator remains.

Reproduce on a clean committed checkout with `scripts/poststop-differential.py`,
`--reference`, `--output` and `--scenario backpressure` or `--scenario eof`;
`cargo test --locked --test hooks` exercises production adapter regressions.

Open gaps include concurrent callback attribution, descendants/process groups,
spawn and uninterruptible kernel waits, all hook phases, rootful/other
architectures and complete OCI conformance. Poststop warning/continuation
conflict remains #35. No complete parity claim follows from these cases.
