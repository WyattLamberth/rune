# OCI support and evidence for the Podman alpha

Target: Linux x86_64, rootless Podman, cgroup v2. Accepting a schema version is
not complete support for every field in that version. This matrix records the
initial tested subset; expand it with spec-pinned acceptance cases before any
conformance claim.

| Area | Current behavior | Evidence / remaining obligation |
| --- | --- | --- |
| Version | Explicit 1.0.0/1.0.1/1.0.2/1.1.0/1.2.0/1.2.1/1.3.0 acceptance | Unit tests; schema acceptance only |
| Config/process input | 4 MiB maximum; unknown fields and NUL arguments/environment rejected | Parser regressions |
| Configuration lifetime | Owned snapshot passed to create/exec; stored create config used for later exec | Snapshot regression; hostile filesystem races still need coverage |
| IDs and state | Bounded single path component; owned private state root; per-ID locks and atomic commits | Lock, symlink, stale identity and state-write regressions |
| Lifecycle | create/start/state/kill/delete, pause/resume, exec | Rootless Podman smoke; raw create barrier and crash recovery remain separate tests |
| Process | Workload output, environment/cwd/user delegated to libcontainer | Output/nonzero exit smoke; expanded user/rlimit cases open |
| Namespaces | Native Linux adapter honors the bundle | PID/cgroup and default rootless network smoke; complete namespace matrix open |
| Cgroup v2 | systemd manager; memory and process limits | Actual 64 MiB / 32 process limits and isolated cgroup view asserted |
| Capabilities / privilege | Native capability setup; requested no-new-privileges failures propagate | cap-drop-all and no-new-privileges smoke |
| Seccomp | Native profile applied to init and exec | Actual mode 2 asserted; complete syscall denial/notification matrix open |
| SELinux | Requested process label applied; mqueue inode label; filesystem-specific mount labels | Podman init/exec labels and label-policy regressions; wider host matrix open |
| TTY | Console socket and PTY adapter | Podman -t smoke; interactive resize and attach coverage open |
| Hooks | Delegated to native backend | Spec ordering/failure/timeout conformance not yet established |
| Annotations | Preserved as metadata | Opt-in backend debug logs can include metadata; telemetry absent |
| Intel RDT | Rejected before resource setup | Cleanup contract not implemented |
| CLI gaps | Reject no-new-keyring, ignore-paused, direct exec --tty without process file | Unknown CLI flags fail; standalone run/spec/features pending |
| Other platforms | Rejected platform config | Linux only; aarch64 declared in Nix but execution unverified |

The full resource ledger, replayable kernel-port failures, stale completion
generations and crash recovery remain in GitHub #3/#4/#6. This alpha does not
claim to satisfy those broader acceptance criteria.
