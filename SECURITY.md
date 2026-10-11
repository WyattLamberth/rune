# Security boundary

Experimental software; the first increment has no independent human review or
full OCI conformance. Rune forbids unsafe code in first-party crates. Third-party
libcontainer performs kernel setup and contains unsafe internals; source and
local patches are disclosed in vendor/README.md.

Preserve requested isolation. Parsing unknown fields must error. Requested
SELinux labels must apply before workload execution; mqueue labels use an inode
xattr and invalid cgroup mount options are excluded. Mount failure after entering
a cgroup namespace must not fall back to exposing the host cgroup hierarchy.
Exec retains the container's original security configuration.

State roots are private, owned directories. Container IDs cannot escape them.
Locks reject concurrent operations. State is committed with atomic rename and
synchronization. Stored process start times plus pidfds protect single-process
signals from PID reuse; group signals also recheck cgroup membership after
acquiring process handles. These tests do not prove complete race freedom.

Do not record workload data in telemetry. Debug logging is opt-in and may include
backend paths and workload metadata, so keep it local. Do not publish those logs.
Telemetry is not implemented yet.

Remaining obligations include complete cleanup fault injection, crash recovery,
hook/notification conformance, privileged containers, hostile filesystem races,
rootful/kernel-version matrix testing, advisory scanning and independent review.
Never weaken SELinux, seccomp or namespaces just to make an integration test pass.
