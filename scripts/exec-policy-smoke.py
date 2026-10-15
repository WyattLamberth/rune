#!/usr/bin/env python3
"""Direct runtime exec policy checks with private disposable Podman resources."""
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
from smoke_cleanup import finish_resources, run_bounded, public_error
from smoke_scope import make_runtime, summary
import tempfile

IMAGE = "docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15"


def main():
    if sys.flags.optimize:
        raise SystemExit("validation requires Python assertions enabled")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--image", default=IMAGE)
    parser.add_argument("--runtime-kind", choices=["rune", "runc"], default="rune")
    parser.add_argument("--results", type=Path, help="write aggregate case outcomes without workload output")
    args = parser.parse_args()
    runtime = args.runtime.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="rune-exec-policy-"))
    runtime, scope_trace = make_runtime(runtime, root / 'state')
    base = ["podman", "--root", str(root / "store"), "--runroot", str(root / "run"),
            "--storage-driver", "vfs", "--runtime", str(runtime),
            "--runtime-flag", f"root={root}/state"]
    cid = None
    errors = []
    case_ids = ["direct-policy", "explicit-overrides", "process-file-policy", "process-file-conflict", "init-identity", "cleanup"]
    cases = {case: "not_run" for case in case_ids}
    current = "direct-policy"

    def call(argv, timeout=45):
        result = run_bounded(base + argv, text=True, timeout=timeout)
        if result.returncode:
            raise RuntimeError(f"{argv[0]} failed with exit {result.returncode}; output withheld")
        return result.stdout.strip()

    def direct(argv, expected=0):
        result = run_bounded(base + ["unshare", str(runtime), "--root", str(root / "state"),
                                      "exec"] + argv, text=True, timeout=45)
        if result.returncode != expected:
            raise RuntimeError(f"direct exec exited {result.returncode}, expected {expected}; output withheld")
        return result.stdout.strip()

    try:
        call(["pull", args.image], timeout=180)
        cid = call(["create", "--network", "none", "--user", "1234:1235", "--workdir", "/tmp",
                    "--ulimit", "nofile=128:128", "--cap-drop", "all", "--security-opt",
                    "no-new-privileges", "--env", "RUNE_EXEC_TEST=original", args.image, "sleep", "120"])
        call(["start", cid])
        info = json.loads(call(["inspect", cid]))[0]
        probe = ('id -u; id -g; pwd; ulimit -n; grep NoNewPrivs: /proc/self/status; '
                 'printf "%s\\n" "$RUNE_EXEC_TEST"; grep CapEff: /proc/self/status; '
                 'cat /proc/self/attr/current')
        output = direct([cid, "/bin/sh", "-c", probe]).splitlines()
        assert output[:4] == ["1234", "1235", "/tmp", "128"], "identity/cwd/limit policy changed"
        assert output[4].split()[1] == "1", "no-new-privileges lost"
        assert output[5] == "original", "environment lost"
        assert int(output[6].split()[1], 16) == 0, "capability policy changed"
        expected_label = info.get("ProcessLabel")
        if expected_label:
            assert output[7].rstrip("\0") == expected_label, "SELinux exec label changed"
        cases[current] = "pass"
        print("PASS direct exec user/group/cwd/rlimit/environment/capabilities/privilege policy", flush=True)
        current = "explicit-overrides"
        overridden = direct(["--env", "RUNE_EXEC_TEST=first", "--env", "RUNE_EXEC_TEST=last",
                             "--cwd", "/", cid, "/bin/sh", "-c", 'pwd; printf "%s\\n" "$RUNE_EXEC_TEST"'])
        assert overridden.splitlines() == ["/", "last"], "explicit overrides changed"
        cases[current] = "pass"
        current = "process-file-policy"
        process = root / "process.json"
        process.write_text(json.dumps({"user": {"uid": 1234, "gid": 1235}, "cwd": "/tmp",
                                       "env": ["PATH=/bin"], "args": ["/bin/sh", "-c", "ulimit -n; grep NoNewPrivs: /proc/self/status"]}))
        inherited = direct(["--process", str(process), cid]).splitlines()
        expected_nnp = "1" if args.runtime_kind == "rune" else "0"
        parent_limit = call(["unshare", "python3", "-c", "import resource;print(resource.getrlimit(resource.RLIMIT_NOFILE)[0])"])
        expected_limit = "128" if args.runtime_kind == "rune" else parent_limit
        assert inherited[0] == expected_limit and inherited[1].split()[1] == expected_nnp, "process-file policy fallback changed"
        cases[current] = "pass"
        current = "process-file-conflict"
        direct(["--process", str(process), "--cwd", "/", cid], expected=1 if args.runtime_kind == "rune" else 0)
        cases[current] = "pass"
        print("PASS explicit overrides, process-file inheritance and conflicting-option rejection", flush=True)
        after = json.loads(call(["inspect", cid]))[0]
        current = "init-identity"
        assert after["State"]["Pid"] == info["State"]["Pid"], "exec changed init identity"
        cases[current] = "pass"
    except Exception as error:
        cases[current] = "fail"
        errors.append(public_error(error))
    finally:
        cleanup_errors = len(errors)
        cleanup_report = finish_resources(root, runtime, call, errors,
            Path(__file__).resolve().parents[1] / 'work/smoke-recovery')
        cases["cleanup"] = "pass" if len(errors) == cleanup_errors else "fail"
    if args.results:
        args.results.write_text(json.dumps({"runtime_kind": args.runtime_kind, "cases": cases, "cleanup": cleanup_report, "runtime_scope": summary(scope_trace),
            "known_differences": ["process-file-policy", "process-file-conflict"],
            "gaps": ["apparmor-denial", "rootful", "architectures", "complete-conformance"]}, indent=2) + "\n")
    if errors:
        raise ExceptionGroup("exec policy validation or cleanup failed", errors)
    print("PASS init identity and disposable resource cleanup", flush=True)
    print("AppArmor denial requires an AppArmor-equipped test environment; no such claim here.")


if __name__ == "__main__":
    main()
