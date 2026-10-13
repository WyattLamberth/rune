# rune

An experimental Linux OCI runtime written in Rust, following rmux's engineering
principles. Code is disposable; engineering time is disposable. Correctness,
isolation, usability and measured performance decide what stays.

AI-generated implementation under WyattLamberth's product direction. No human
review or full OCI conformance is implied. Independent project, not an OpenAI product.

## Current increment

Native Rust Linux adapter using the youki `libcontainer` library, not a runtime
executable wrapper. Supports create/start/state/kill/delete, exec, pause and resume.
Rune's own crates forbid unsafe code; the backend dependency has unsafe internals.
See [adapter provenance and patches](vendor/README.md).

Rootless Podman integration is the first target. Tested locally with Podman 5.8.7
on Linux x86_64 and cgroup v2: run, nonzero exits, memory/PID limits, SELinux,
seccomp, no-new-privileges, exec, TTY, pause/resume, kill, default networking
and cleanup. Broader rootful, network,
volume, architecture and OCI conformance coverage remains open.

## Build and try with Podman

With Rust 1.96.1 and libseccomp development libraries:

```sh
cargo build --release --locked
# If only the installed libseccomp runtime library is available on x86_64:
bash scripts/build-local.sh --release
# Or with Nix installed:
nix develop -c cargo build --release --locked

podman --runtime "$PWD/target/release/rune" run --rm   docker.io/library/alpine:3.23 echo hello-from-rune
podman --runtime "$PWD/target/release/rune" run --rm -it   docker.io/library/alpine:3.23 sh
```

Select the runtime per invocation. Existing containers retain their configured
runtime. `scripts/podman-smoke.py` tests with isolated temporary storage and
default SELinux/seccomp settings, without changing Podman defaults.

```sh
python3 scripts/podman-smoke.py --runtime "$PWD/target/release/rune"
python3 scripts/exec-policy-smoke.py --runtime "$PWD/target/release/rune"
cargo test --workspace --locked
cargo clippy --workspace --locked --all-targets -- -D warnings
RUNE_TEST_SEED=42 RUNE_TEST_STEPS=100000 cargo test --locked --test universe
```

## Design and limitations

Dependency-free deterministic policy core; native Linux adapter and OCI parser
outside it. Bounded input, validated IDs, nonblocking per-container command locks,
configuration snapshots, atomic state writes and stable-process signal handles.
The seeded simulator currently covers policy dispatch and failure propagation,
not kernel setup, complete rollback, IO scheduling or crash recovery.

State defaults to `$XDG_RUNTIME_DIR/rune`, including when Podman enters its user
namespace; otherwise root uses `/run/rune`. Use `--root` to select a private store.
Lock files survive deletion so concurrent callers always lock the same inode;
long-term lock-file reclamation is not implemented yet.

Direct exec retains the stored create process's user, cwd, AppArmor and resource
policy, then applies explicit CLI overrides. Process files inherit omitted
security/resource fields; combining `--process` with process overrides or a
command errors explicitly. Rune preserves the create snapshot instead of
rereading a changed bundle. Full direct-exec parity remains tracked in #16/#13.

Accepted OCI schema versions are listed in code, including Podman's 1.2.1. This
is not a claim that every field is implemented. Unknown fields are rejected;
Intel RDT, `--no-new-keyring`, direct `exec --tty` without a process file, and
`--ignore-paused` are rejected. Full feature negotiation, standalone run/spec,
reproducible performance baselines, telemetry, and release artifacts remain backlog work.

[OCI support matrix](docs/oci-support.md) · [Architecture](docs/architecture.md) · [security boundary](SECURITY.md) ·
[runc parity baseline and gaps](docs/parity/README.md) · [Development log](docs/development-log.md) ·
[GitHub backlog](https://github.com/WyattLamberth/rune/issues) ·
[Linear delivery](https://linear.app/rmux/project/rune-4971526b84f3)
