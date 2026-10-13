# Bounded NUMA preflight validation — 2026-10-12

AI-generated implementation and review, GitHub #23 / RMU-35. Exact clean
implementation `ce207b9df26a2e8f3d8ff8bbed8eb738505bc425`.

| Check | Result |
| --- | --- |
| Formatting / warnings-denied locked all-targets Clippy | Pass |
| Locked workspace tests | 29 pass |
| Architecture / source inventory integrity | Pass |
| Standalone label tests | 2 pass |
| Rootless NUMA preflight differential | Five matching groups: safe malformed-policy rejection/cleanup, init, both exec forms and cleanup |
| Failed-create rootless suite | Missing/non-executable failure and runtime rollback pass |
| General rootless Podman kernel suite | Pass, including runtime cleanup |
| Dependency patch reproduction | Pass against checksum-pinned package |

```sh
python3 scripts/exec-differential.py --suite numa-preflight \
  --reference work/reference-runc/runc.amd64 --output work/numa-bounds-differential.json
```

The native production parser uses the fixed CPU-set ID ceiling found in pinned
runc (1023 on the tested Linux amd64 reference) and the existing 4 MiB input
bound. It merges intervals before expanding unique IDs. Tests reject u32-wide
ranges, high/overflow IDs, oversized text, malformed numbers and reversed ranges;
repeated full ranges remain bounded. Exhaustive small-range comparisons verify
set semantics. Eight adapter CLI negatives leave no container ownership records,
and stored exec preflight checks the same production validator.

Kernel negatives use safe MPOL_LOCAL-with-nodes, absent MPOL_BIND nodes and
node 1024. Both runtimes fail and clean up. Giant ranges are exercised only
through the new bounded parser/adapter, never the old vulnerable implementation.
Valid MPOL_LOCAL mappings remain observable in init and both exec forms. Private
fixture preparation preserves ordinary rootless Podman isolation. No host node
IDs, paths, workload output or logs are published.

Review inspected reference ToCPUSet and policy validation, range/mask bounds,
preflight integration before container/tenant acquisition, failure records,
fixture/cleanup boundaries, prior decisions and Linear. No unsafe/dependency or
lockfile changes. This establishes a specific bounded parser and kernel subset.
Text acceptance differences, all mode/flag combinations, denied features, rootful,
other architectures and concurrency remain explicit gaps. No full NUMA or
complete runc parity claim; #23/#22/#14 retain broader acceptance work.
