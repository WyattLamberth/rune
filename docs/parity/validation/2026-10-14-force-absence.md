# Forced absence and live cleanup faults — 2026-10-14

AI-generated implementation/review, GitHub #34 / RMU-39 and #17 / RMU-33.
Exact clean implementation `8d8b4a46419463a9daa29374af99fcd543543f2b`.

| Check | Result |
| --- | --- |
| Formatting / warnings-denied locked all-targets Clippy | Pass |
| Locked workspace tests | 30 pass |
| Seeded production policy replay | Default seeds plus seed 42, 100000 steps, bounded trace on failure |
| Architecture / inventory integrity | Pass |
| Signed-reference standalone CLI/filesystem comparison | Complete forced absence matches; partial ownership is a known difference |
| Live private rootless metadata faults | Missing/malformed/FIFO reject delete without changing state bytes or init identity |
| Restore/retry and owned frontend cleanup | Pass |
| General signed-reference kernel suite | Matching full assertions and strict cleanup |

```sh
python3 scripts/delete-absence-differential.py --reference work/reference-runc/runc.amd64 \
  --output work/delete-absence.json
python3 scripts/cleanup-metadata-smoke.py --runtime target/debug/rune \
  --results work/live-cleanup.json
LIBSECCOMP_LIB_PATH="$PWD/work/native-lib" RUNE_TEST_SEED=42 RUNE_TEST_STEPS=100000 \
  cargo test --locked --test universe
```

Absent is an internal production-policy observation, not a new OCI state output.
Force deletion of a completely missing directory produces Noop and invokes no
adapter; ordinary deletion still errors. The adapter validates IDs and takes
the per-ID lock before observing ENOENT. Other I/O failures and existing partial,
corrupt, file or symlink ownership remain errors. The core stays deterministic
and dependency-free with unsafe forbidden. No lockfile/dependency changes.

The policy simulator exercises the production dispatch with seed, step and
bounded replay trace; it does not prove kernel isolation/concurrency. Standalone
absence tests exercise production CLI/filesystem behavior without containers.
The live suite uses an actual owned rootless container, intentionally damages
only its cleanup metadata, confirms original state bytes/init liveness, restores
metadata and retries native deletion. Both workload and frontend cleanup succeed.
No global defaults or storage changes; all fixture state roots remain private.

Pinned runc v1.5.2 removes a partial synthetic directory without state.json and
returns forced success. Rune retains unknown ownership and returns an error.
That explicit safety/compatibility difference remains unresolved until a
compliant acquisition ledger/recovery design exists. No claim of full deletion
parity, crash recovery, concurrency or complete resource-failure coverage.

Review read core/adapter history, pinned delete.go, transition/no-effect policy,
seeded and deadline-protected CLI regressions, ID/path checks, live fault/retry
results and retained development failure evidence. Earlier failed frontend
cleanup was recovered; no retained resource locator remains. #35 separately
tracks poststop warning/continuation/order disagreement between spec and baseline.
