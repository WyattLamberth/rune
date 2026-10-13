#!/usr/bin/env python3
"""Run a focused exec kernel suite against rune and the pinned signed runc artifact."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REFERENCE_SHA = "599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488"
REFERENCE_REVISION = "29dd3dc2b13b4123162e5fe132504bb4b15569f1"


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", choices=["exec", "numa", "numa-preflight"], default="exec")
    parser.add_argument("--reference", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--allow-dirty", action="store_true", help="development only; record no validated rune revision")
    args = parser.parse_args()
    reference = args.reference.resolve(strict=True)
    if sha256(reference) != REFERENCE_SHA:
        parser.error("reference binary differs from the pinned amd64 release artifact")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    dirty = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=ROOT, text=True)
    if dirty and not args.allow_dirty:
        parser.error("commit changes before exact-revision validation, or use development --allow-dirty")
    subprocess.run(["bash", "scripts/build-local.sh"], cwd=ROOT, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"], cwd=ROOT))
    rune = Path(metadata["target_directory"]) / "debug/rune"
    suite = "exec-policy-smoke.py" if args.suite == "exec" else "numa-policy-smoke.py"
    report = {"format": 1, "suite": args.suite, "rune_revision": None if dirty else head,
              "development_base": head if dirty else None,
              "rune_binary_sha256": sha256(rune), "runc_revision": REFERENCE_REVISION,
              "runc_binary_sha256": REFERENCE_SHA,
              "runc_release": "v1.5.2", "runc_artifact": "runc.amd64",
              "reference_source_build_reproduced": False,
              "rustc": subprocess.check_output(["rustc", "--version"], cwd=ROOT, text=True).strip(),
              "cargo_lock_sha256": sha256(ROOT / "Cargo.lock"),
              "suite_sha256": sha256(ROOT / "scripts" / suite),
              "image": "docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15",
              "scope": "Rootless " + args.suite + " exec kernel subset; not complete conformance or parity.",
              "runtimes": {}, "comparisons": {}}
    with tempfile.TemporaryDirectory(prefix="rune-exec-differential-") as directory:
        for kind, binary in (("runc", reference), ("rune", rune)):
            output = Path(directory) / (kind + ".json")
            command = [sys.executable, str(ROOT / "scripts" / suite),
                       "--runtime", str(binary), "--runtime-kind", kind,
                       "--results", str(output)]
            if args.suite == "numa-preflight":
                command.append("--preflight")
            result = subprocess.run(command, cwd=ROOT, capture_output=True)
            # Do not include subprocess logs, container IDs, paths or workload
            # output in the distributable aggregate record.
            report["runtimes"][kind] = {"exit_code": result.returncode,
                                       "results": json.loads(output.read_text()) if output.exists() else None}
    left = report["runtimes"]["runc"]["results"]
    right = report["runtimes"]["rune"]["results"]
    if left and right:
        for case in sorted(set(left["cases"]) | set(right["cases"])):
            passed = left["cases"].get(case) == right["cases"].get(case) == "pass"
            report["comparisons"][case] = ("known_difference" if case in right.get("known_differences", []) else "matching_assertions") if passed else "validation_gap"
    after_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    after_dirty = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=ROOT, text=True)
    changed = after_head != head or after_dirty != dirty
    if changed:
        report["rune_revision"] = None
        report["source_changed_during_validation"] = True
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"rune_revision": report["rune_revision"], "comparisons": report["comparisons"]}, sort_keys=True))
    if changed or any(value["exit_code"] for value in report["runtimes"].values()):
        raise SystemExit("differential validation failed; aggregate record preserves gaps")


if __name__ == "__main__":
    main()
