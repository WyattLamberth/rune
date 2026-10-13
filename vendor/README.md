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
- Preserve SELinux and seccomp configuration on exec, provide exec container
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
