# rune workflow

AI-generated implementation under WyattLamberth's product direction. Never imply human review.
Use Git identity Codex <codex@openai.com>. GitHub issues are the implementation backlog;
Linear organizes delivery. Use issue-linked branches and PRs and exact-revision validation.

Forbid unsafe code in rune crates. Maintained dependencies may contain unsafe internals;
this is a documented trust boundary. Keep the core dependency-free and deterministic.
Linux, filesystem, clocks and processes belong in adapters. Simulations exercise production
policy and report seed, step and a bounded replay trace. Explicitly distinguish policy
simulation from kernel isolation and concurrency testing.

Never weaken isolation to make a test pass. Unsupported CLI/features must error.
Use disposable, uniquely named test containers; never reset Podman storage, migrate its
runtime, or alter global defaults. Bound input, validate IDs before joining state paths,
and preserve cleanup errors. Never publish host details, workload content, logs or secrets.

Run formatting, warnings-denied Clippy, architecture checks and relevant tests.
Keep Cargo.lock and flake.lock committed. Measure performance claims with reproducible
workloads. Telemetry is opt-in aggregate data without container IDs or workload contents.

Core tenets: code is disposable; engineering time is disposable. Replace designs freely
when correctness, isolation, usability or measured performance improves. Do not preserve
code to justify sunk effort. Safety and validation remain mandatory.
