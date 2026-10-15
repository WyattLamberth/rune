# libcontainer adapter provenance

`libcontainer` 0.7.0, Apache-2.0, from the youki project.
Upstream packaged revision: `94ba653efbb180ce04650f6ae01a8e6bc8f96d92`.
Crates.io archive SHA-256: `2c28770cf8fb3acca0d168a10f5d9f8333d304af0b2ff5654029822d222d19f0`.

This is a maintained third-party dependency with its existing unsafe syscall
internals. Rune's own crates forbid unsafe code. The rune patch adds no unsafe
blocks. No other runtime executable is invoked.

Local changes:

- Apply mqueue SELinux labels using an inode xattr, not an invalid mount option;
  requested labels fail closed. Skip invalid cgroup context mount options.
- Refuse a fallback to the host cgroup root after entering a cgroup namespace.
- Apply requested SELinux process labels before announcing creation readiness.
- Preserve NUMA memory policy, SELinux and seccomp configuration on exec, provide exec container
  metadata to the seccomp handshake, and retain the init PID during exec.
- Feed validated spec/process snapshots directly to the builders and persist
  the full create configuration for subsequent exec rather than rereading bundles.
- Store process start time, refuse stale identities, and signal with pidfds.
- Propagate failures to apply requested no-new-privileges settings.
- Commit state through a synchronized temporary file and atomic rename.
- Require bounded regular cleanup metadata before deletion effects, reject
  symlink/nonregular metadata, and propagate cleanup failures while retaining state.

These are local patches, not changes accepted by upstream. The minimal source
patch is in `docs/libcontainer.patch`. Broader syscall/fault/conformance coverage
and independent review remain required. The supplied policy label regression
can run with `rustc --test vendor/libcontainer/src/rootfs/selinux.rs`.

Create rollback now retains ownership records if cgroup or RDT teardown fails.
The shared production filesystem finalizer is exercised by workspace fault/retry
tests; it does not establish kernel resource failure or concurrency coverage.

NUMA policy uses a bounded production node parser with merged duplicate ranges,
a fixed CPU-set-sized node-ID ceiling matching pinned runc, and the existing 4 MiB configuration text
limit. Create and stored exec snapshots use the same policy preflight before
container acquisition or tenant setup. These bounds do not prove exhaustive NUMA semantics.

Delete removes runtime ownership state after successful kernel/resource cleanup
and before poststop hooks, matching pinned runc and OCI deletion timing. Hook
errors remain observable; pinned reference/spec continuation and warning
semantics are explicitly unresolved under #35.

Hook stdin is delivered through the existing pipe in nonblocking mode, under the
same deadline as exit waiting. The writer is closed for EOF consumers. A single
Child owner kills and reaps on delivery/wait errors and retains cleanup failures;
negative/unrepresentable deadlines fail before spawning. See issue #38. No
unsafe code or new dependency is introduced; libc/nix remains a maintained
adapter dependency trust boundary. Descendant process groups are not proven
cleaned up by this direct-child change.

Requested PID files are published through a synchronized random temporary file
and atomic rename after readiness, before returning create or releasing exec.
Failure terminates/reaps only a proven owned child through a pidfd; incomplete
process/temp/socket rollback retains private recovery ownership and prevents
further lifecycle commands. Automatic reconciliation of that exceptional record
is not implemented. Full syscall-fault/crash/concurrency coverage remains #31/#17.

Regenerate the minimal source patch with `python3 scripts/backend-patch.py --write`
and verify it with `--check`. The helper checks the upstream crate checksum and
uses its local Cargo cache archive, or an explicit `--archive PATH`.
