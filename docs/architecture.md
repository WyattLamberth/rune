# Architecture

`rune-core` contains deterministic command planning and production dispatch.
Invalid transitions never invoke adapters; adapter failures remain failures.
The CLI uses this dispatcher for lifecycle commands, with observed state supplied
by the native Linux adapter. The simulator calls the same dispatcher, injecting
failures and checking an independent command-policy oracle with bounded traces.

`src/validation.rs` bounds OCI config and exec process documents to 4 MiB,
rejects unknown fields, validates released version strings and basic process
requirements, and returns owned typed snapshots. Builders receive those snapshots
without rereading caller-controlled configuration. The backend persists the full
create config for exec, retaining original namespaces and security configuration.

`src/exec_policy.rs` composes exec processes without filesystem or kernel effects.
Direct exec begins with the bounded stored create process; CLI overrides change
only specified policy. Process files retain explicit values and inherit omitted
container security/resource policy. Ambiguous process-file/override combinations
are rejected. Environment duplicates resolve to the last assignment so executable
lookup and the installed environment agree. The Linux adapter passes the complete
validated snapshot to the backend rather than its default process builder.

`src/linux.rs` composes the native Rust library, validates IDs for every command,
serializes commands with a private per-container nonblocking flock and dispatches
lifecycle effects through the core. The dependency manages process setup,
namespace membership, filesystem isolation, cgroups, hooks and state storage.
The local dependency patch is explicitly documented and licensed separately.

Deletion requires readable, bounded regular cleanup metadata before signals or
state changes. Corrupt/missing metadata and resource/hook teardown failures retain
ownership records for diagnosis and retry. This deliberately fails incomplete
recovery rather than guessing that no resources exist; the full recovery ledger
and metadata reconstruction protocol remain open.

The current design delegates lifecycle/resource realization to libcontainer.
It is a real native adapter but not the complete resource-ledger state machine
planned in the backlog. Full deterministic setup/rollback simulation, generation
acknowledgements and crash recovery remain separate work. Safety and performance
evidence can justify replacing this adapter or reshaping the architecture.

## Validation layers

- Exhaustive policy, parser bounds, unknown fields, snapshot, ID and locking tests.
- Owned subprocess tests for stale identity rejection, pidfd signaling and atomic
  state replacement. These are not kernel container-isolation tests.
- Seeded production dispatch and failure propagation replay.
- mqueue/cgroup SELinux label policy regression tests in the patched dependency.
- Rootless Podman smoke tests with isolated storage, checking actual workload
  exits, SELinux labels, seccomp, cgroup limits, exec, TTY and lifecycle cleanup.

Nix pins are inherited from rmux. x86_64-linux and aarch64-linux are declared;
architecture declarations do not establish successful builds on both machines.
