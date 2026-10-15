# Cleanup harness evidence — 2026-10-13

AI-generated implementation/review, GitHub #17 / RMU-33. Corrected clean
implementation `7a609bc99bae0da2c8be0d89a5d55b5cc98289e8`.

| Check | Result |
| --- | --- |
| Formatting / warnings-denied locked all-targets Clippy | Pass |
| Locked workspace tests / cleanup control-filesystem tests | 29 / 8 pass |
| Architecture / source inventory integrity / label tests | Pass / pass / 2 pass |
| Pinned direct exec differential | Four matching groups, two known differences retained |
| NUMA preflight differential | Five matching groups |
| Failed-create rootless suite | Pass, including resource cleanup |
| Broader Podman kernel differential | Validation gap: runc leftover state; rune completes in this run |

The broader comparison reaches the final cleanup assertion on the pinned signed
runc reference, with one runtime state record left after Podman's registry is
empty. The harness fails and preserves the owned tree. Subsequent native state
query reports stopped, direct forced delete succeeds, and private tree removal
is confirmed. Recovery is recorded separately; it does not turn the failed
comparison into a pass. No private recovery locator remains after recovery.

```sh
python3 -m unittest discover -s scripts -p test_smoke_cleanup.py
python3 scripts/exec-differential.py --suite podman \
  --reference work/reference-runc/runc.amd64 --output work/general-differential.json
```

The shared helper enumerates only the uniquely generated private Podman store,
requests full IDs and validates them before removal. Bounded diagnostic reads use
anchored directory descriptors, no-follow and nonblocking opens. Operation and
cleanup failures survive together. Uncertain cleanup retains ownership and a
private mode-0600 recovery locator; aggregate records exclude paths, IDs, logs
and workload contents. Final tree absence is confirmed after namespace cleanup.

The first exact harness revision `24387ac95edce4485816dd5cfd8e56b1c115e5a4`
failed because its enumeration requested default shortened Podman IDs, conflicting
with the full-ID validator. This was corrected with --no-trunc and a regression
test. That failed run is not runtime defect evidence. Earlier dirty broader
development runs also retained runc and rune state; no exact candidate revision
is attributed to those results.

Installed Podman reports 5.8.7. Investigation reads its upstream tag source at
`c593b672bf3db1173aebea565ebf1a724ea196dc`; its local build is not reproduced.
ContainerRun logs removal errors while returning the workload report without
propagating the removal error (`pkg/domain/infra/abi/containers.go:1300`).
Runtime removal proceeds to remove registry records after cleanup errors
(`libpod/runtime_ctr.go:979`). This can explain why expected workload exit codes
and an empty registry are insufficient cleanup evidence. It does not establish
the exact underlying runtime delete failure or resolve the original rune case.

Review inspected source/history/decisions, failure aggregation, anchored bounded
reads, cleanup ownership, external CLI ID contract, private recovery and source
provenance. Control/filesystem fault tests are distinct from container isolation
and concurrency evidence. No runtime policy, unsafe code, dependency, lockfile or
isolation change. Next: capture bounded native delete request/results with the
pinned frontend to distinguish missing calls, concurrency and actual teardown
errors. Keep #17 and broad parity acceptance open.
