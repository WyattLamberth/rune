# First exact-revision exec differential result

AI-generated validation/review for GitHub #13 / RMU-30.
Tested rune revision: `e51c0f46d6a9a1a13fb2a41ddc9011844a2300b1`.
The clean-revision runner built rune with locked dependencies and checked source
stability through validation. [Aggregate record](2026-10-12-exec-differential.json)
contains source/artifact/binary/lockfile/suite digests and classifications; no
container IDs, host identities, paths, logs or workload output are included.

Both runtime suites passed expected assertions and cleanup. Four case groups
match: direct process policy, explicit overrides, init identity and cleanup.
Two case groups are **known differences**, not parity: omitted process-file
privilege/file-limit policy, and ignored versus rejected process-file overrides.
The development failure that revealed runc's different file-limit behavior was
corrected through measurement, not by changing rune isolation.

At this exact revision, formatting, warnings-denied locked all-targets Clippy,
19 workspace tests, architecture and inventory checks passed. Reference preparation
verified pinned checksum, signer, source revision and reported build provenance.
The reference is the signed upstream release artifact, not a reproduced source
build. See [methodology](../exec-differential.md) for reproducible commands and
trust/coverage limits.

AI-generated review included subprocess timeouts and cleanup-error preservation,
dirty-source refusal, revision invalidation, reference hash/signature pins,
aggregate output privacy, known-difference handling, prior failed expectations,
commit history/development log, and accurate Linear status. No human review or
exhaustive semantic coverage is implied. #13 remains open for source-built and
conformance tooling plus broader kernel/environment coverage; #16 remains open
for AppArmor denial and unresolved compatibility differences; #17 remains open
for intermittent cleanup investigation. Final PR head checks are recorded on
the PR because this evidence commit follows the tested implementation revision.
