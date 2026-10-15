# Private runtime-root propagation — 2026-10-14

AI-generated implementation/review, GitHub #17 / RMU-33. Clean implementation
`26074b5492ed2f7ff28ce31c013bb36c8cad1623`; test deadline/reaping follow-up
`22232ce` is separately committed. Final head checks belong on the PR.

| Check | Result |
| --- | --- |
| Formatting / warnings-denied locked all-targets Clippy | Pass |
| Locked Rust / fixture-control tests | 29 / 14 pass |
| Architecture / inventory / label tests | Pass / pass / 2 pass |
| Pinned direct-exec differential | Four matching groups, two existing known differences |
| NUMA preflight differential | Five matching groups |
| General Podman differential | Matching full suite assertions and strict cleanup for both runtimes |
| Failed-create suite | Pass, including cleanup |

The general comparison records eight native creates and deletes for each
runtime. One delete request per runtime omits its root argument. The fixture
binds the expected private root, and both leave no runtime state or retained
resource locator. Earlier development comparisons also observed omitted-root
deletes with successful scoped cleanup. Counts are observations, not performance
claims or exhaustive concurrency proof.

```sh
python3 -m unittest discover -s scripts -p 'test_smoke_*.py'
python3 scripts/exec-differential.py --suite podman \
  --reference work/reference-runc/runc.amd64 --output work/scoped-general.json
```

Pinned Podman source `c593b672bf3db1173aebea565ebf1a724ea196dc` builds its conmon
exit command with the runtime executable but omits per-invocation runtime flags
(`pkg/specgenutil/util.go`, CreateExitCommandArgs). Its ContainerRun path can log
removal errors and still return workload status; removal can proceed to discard
registry records after cleanup errors. The recorded omitted-root requests and
successful bound-root cases establish a concrete fixture routing defect consistent
with that source path. The original unrecorded rune failure cannot be attributed
retrospectively with certainty. Installed Podman reports 5.8.7; its build is not
reproduced. The signed runc/source/spec baseline is unchanged.

The generated launcher binds an explicit private state root, rejects conflicting
or duplicated root values and expired/non-owned scopes, and invokes the native
binary with execv. It preserves PID, descriptors and stdio; a deadline-protected
owned process/descriptor test exercises that contract. There is no additional
parent process and rune still executes its native Rust backend. Root binding is
fixture configuration and makes callbacks obey the same private scope as create.
No global defaults, runtime policy or isolation controls are changed, and no
cleanup assertion is relaxed or delayed. All four fixtures use the same binding.

Trace output contains only operation/boolean categories, bounded to 64 KiB with
regular/no-follow descriptor checks. Published aggregate counts contain no IDs,
paths, workload contents or logs. The trace records requests, not individual
command exit results; workflow/state checks provide the outcome evidence.

Review reconciled completed PR #26 output bounds with PR #28 cleanup evidence,
retaining both histories. The other active chat handles #27 nonregular JSON in
its own worktree, which this increment does not modify. No unsafe, dependency or
lockfile changes. Real cgroup/hook failure injection, interrupted acquisition,
crash recovery, concurrent native operations, architectures and complete parity
remain open under #17/#13. Earlier failure records remain failures; later passes
do not erase them.
