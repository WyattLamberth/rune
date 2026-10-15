# Native alpha integration — 2026-10-14

AI-generated integration and review under GitHub #30 / Linear RMU-37. This
integration moves the actual implementation toward main; earlier branch publication
and validation were not delivery. Main previously contained only documentation
at a9ba97984ccc2c9fedb538e7a10c41070b5df91a.

The combined review preserves implementation ancestry from the eleven stacked
PRs through c614c12 and the cleanup-evidence fork at 763740e. Runtime code is
unchanged by the integration; test changes add missing init seccomp/domain
assertions and explicit conditional SELinux outcomes. Main's actual merge SHA
and post-landing checks will be recorded on GitHub #30 and the integration PR.
On resumption inspect those authoritative records before repeating integration.

| Evidence | Exact revision / result |
| --- | --- |
| Format / warnings-denied locked all-targets Clippy | db3685c91bc699a850a61b96d64c1f4939f21235: pass |
| Locked Rust tests / Python cleanup tests / labels | Same revision: 31 / 10 / 2 pass |
| Architecture and inventory integrity | Same revision: pass; zero fully verified parity entries |
| Dependency patch reproduction | 73 Rust/TOML files exactly reproduced from checksum-pinned libcontainer 0.7.0 archive |
| Standalone input differential | Same revision: four matching rejections, documented FIFO policy difference |
| Rootless exec differential | Same revision: four matching groups, two known process-file differences |
| Rootless NUMA preflight differential | Same revision: five matching groups |
| Rootless failed-create rollback | Same revision: both executable failures and cleanup pass |
| Release build and complete rootless alpha smoke | 300fc4c3e35cf99d8533b94533c474fac3c36e84: pass |
| Hosted Rust/Nix | Final integration head required before landing |

The release record pins binary, suite/helper, lockfile, toolchain, frontend and
image provenance. It covers ordinary output and nonzero exits, memory/PID limits,
cgroup namespace, dropped capabilities, no-new-privileges, init/exec seccomp mode,
init SELinux domain and exec exact configured label, TTY, pause/resume, kill/wait,
init identity, default networking and confirmed container/state/tree cleanup.
Both conditional SELinux checks actually ran and passed. Seccomp mode 2 proves
filter installation; these probes are not exhaustive denial/action or MCS tests.
The private fixture leaves no recovery resources. Later passing runs do not close
the earlier retained-state failures under #17; all exact failing records remain.

Combined review read the first-party CLI, Linux adapters, snapshot/process policy,
dependency-free core, production dispatch simulation and owned-process/fault
regressions. It checked ID/lock boundaries, bounds and descriptor verification,
requested isolation, stable signals, ownership retention, local backend patch,
license/provenance, prior decisions/history and delivery context. First-party
unsafe code remains forbidden; maintained dependency internals are a documented
trust boundary. No human review is implied. Whole semantic, rootful, architecture,
crash, concurrency and resource-acquisition-ledger proof remain incomplete.

Known limits stay visible on main: full runc parity is unproven; #27's streamed
input conflict and process-file differences remain; #31's source-only PID-file
failure defect is unresolved; #17's original cleanup cause and recovery/concurrency
acceptance remain. This is an experimental usable command subset. Integration
must not close those broader acceptance issues or claim production certification.
