# Initial exec differential kernel slice (#13 / RMU-30)

This runner covers a narrow rootless exec policy slice using private Podman VFS
storage and a pinned image. It invokes both runtimes directly inside Podman's
user namespace and compares equivalent assertions, with runtime-dependent labels
checked against each container's requested label. It does not run the upstream
integration suite or OCI conformance tools, and does not establish complete parity.

## Reference provenance

The reference is upstream's signed `runc.amd64` artifact for v1.5.2, corresponding
to source `29dd3dc2b13b4123162e5fe132504bb4b15569f1`. SHA-256:
`599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488`.
Detached signature signer fingerprint:
`C2428CD75720FACDCF76B6EA17DE5ECB75A1100E`.
Trust anchor: public keyring in the clean pinned source checkout. Verification
uses a disposable GPG home; it does not alter global trust settings. This proves
artifact identity/signature against that anchor, not an independently certified
key identity or reproduction of the upstream build.

Reported build: Go 1.26.8, libseccomp 2.6.1, libpathrs 0.2.6, OCI spec 1.3.0.
Source-built reference/toolchain reproduction, other architectures and feature
variants remain #13 obligations. This artifact is a test reference only; rune
continues to execute natively through its Rust adapter.

## Run

Requires `gh`, GPG, Podman, Python 3.11+, the locked rune build environment and
the clean pinned runc checkout prepared by the baseline instructions:

```sh
python3 scripts/prepare-runc-reference.py --source work/parity-upstream/runc
python3 scripts/exec-differential.py --reference work/reference-runc/runc.amd64 --output work/exec-differential.json
```

The runner refuses an unrecognized reference digest and dirty rune source. It
builds rune with locked dependencies, records the exact commit, binary digests,
Rust compiler, lockfile/suite digests and image pin, then runs both suites with
isolated resources. Source changes during validation invalidate the revision.
`--allow-dirty` is development-only and deliberately records no validated revision.
Aggregate records omit host identities, container IDs, paths, logs and workload
output. Each subprocess in the suite has a timeout and cleanup errors accumulate
with validation errors; the runner lets the suite's cleanup finish.

## Current assertions and gaps

Matching assertions: direct-exec user/group, cwd, file limit, environment,
capabilities and no-new-privileges; explicit env/cwd overrides; unchanged init
identity; private resource/state cleanup. Cleanup pass is for the observed cases,
not complete race freedom or resolution of intermittent #17.

Known differences remain separate from matching assertions:

| Case | Rune | Pinned runc observed behavior |
| --- | --- | --- |
| Process file omits privilege/file-limit settings | Retains original no-new-privileges and file limit | Omitted no-new-privileges decodes false; absent file-limit setting retains exec parent's limit in the tested path |
| Process file combined with cwd override | Explicit error | Ignores cwd override and executes the process document |

The file-limit expectation is measured relative to the exec parent in the
isolated test namespace, without publishing that limit. An initial source-only
expectation that runc would reapply its container file limit was incorrect; the
differential development run caught it. Do not reinterpret a known difference
as parity. Investigate compliant semantics under #16/#12 rather than weakening
original policy to make a comparison pass.

Open coverage: actual AppArmor denial, broader SELinux/seccomp denial and feature
cases, rootful configurations, cgroups/architectures/kernel variants, full OCI
requirements, upstream tests, ecosystem integrations, concurrency, interruption,
rollback/recovery and complete cleanup. Each remains a validation gap. Upstream
helpers use fixed IDs/host assumptions and must be audited before execution.
