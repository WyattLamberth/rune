# Exec policy validation — 2026-10-12

AI-generated implementation validation and review, GitHub #16 / RMU-32.
Exact implementation revision: `3b6b8ab4c936d9cba29961da73de308b87ef75c4`.
The working tree was clean during the implementation checks. The record itself
is committed afterward; final PR head validation is recorded on the PR.

| Check | Result |
| --- | --- |
| Formatting, warnings-denied locked all-targets Clippy | Pass |
| Architecture and offline parity inventory integrity | Pass |
| Locked workspace tests | Pass: 19 tests |
| Standalone vendored label-policy tests | Pass: 2 tests |
| Disposable direct-exec kernel suite | Pass: identity/cwd/rlimit/environment/capability/NNP, overrides, process-file fallback, init identity and cleanup |
| Diagnostic general Podman smoke | Pass: full suite including state cleanup |
| Tracked general Podman smoke | Pass: full suite including state cleanup; only evidence/checkpoint documents changed after the implementation commit |

Reproduce the implementation checks with the existing project-local libseccomp
alias (or development libraries), Rust 1.96.1 and the scripts' pinned image:

```sh
bash scripts/build-local.sh
cargo fmt --all -- --check
LIBSECCOMP_LIB_PATH="$PWD/work/native-lib" cargo clippy --workspace --locked --all-targets -- -D warnings
LIBSECCOMP_LIB_PATH="$PWD/work/native-lib" cargo test --workspace --locked
python3 scripts/check-architecture.py
python3 scripts/parity-inventory.py
python3 scripts/exec-policy-smoke.py --runtime "$PWD/target/debug/rune"
python3 scripts/podman-smoke.py --runtime "$PWD/target/debug/rune"
```

The direct suite invokes the runtime itself inside Podman's rootless user
namespace, using private VFS storage, a uniquely created container and strict
resource/state cleanup checks. It does not change Podman global defaults or
disable SELinux/seccomp. Only aggregate outcomes are recorded. The diagnostic
general suite was an ignored copy of the tracked smoke script with bounded
status/file-category output added before its existing final cleanup assertion;
the successful path and all original checks are unchanged. Earlier uncommitted
cleanup failure remains #17 / RMU-33, with no exact reproduction claimed.

AI-generated review inspected source composition, stored-snapshot/parser bounds,
CLI/backend integration, complete-policy retention regressions, default-free
capability additions, deterministic last-assignment environment resolution,
process-file fallback, explicit rejection of ignored override combinations,
cleanup error preservation, prior history, the development log and Linear state.
No vendor modifications or lockfile/dependency changes were required.

Remaining gaps: actual AppArmor profile denial and security cross-products,
rootful/architecture coverage, full pinned-runc differential validation and
complete concurrency/recovery/cleanup evidence. Process-file plus overrides and
omitted no-new-privileges have explicitly recorded runc compatibility differences;
they need a compliant design and differential evidence. Policy regressions and
kernel subset tests do not establish complete parity. Keep #16 open until its
remaining acceptance evidence exists.
