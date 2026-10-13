#!/usr/bin/env python3
"""Direct runtime exec policy checks with private disposable Podman resources."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile

IMAGE = "docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--image", default=IMAGE)
    args = parser.parse_args()
    runtime = args.runtime.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix="rune-exec-policy-"))
    base = ["podman", "--root", str(root / "store"), "--runroot", str(root / "run"),
            "--storage-driver", "vfs", "--runtime", str(runtime),
            "--runtime-flag", f"root={root}/state"]
    cid = None
    errors = []

    def call(argv, timeout=45):
        result = subprocess.run(base + argv, capture_output=True, text=True, timeout=timeout)
        if result.returncode:
            raise RuntimeError(f"{argv[0]} failed with exit {result.returncode}; output withheld")
        return result.stdout.strip()

    def direct(argv, expected=0):
        result = subprocess.run(base + ["unshare", str(runtime), "--root", str(root / "state"),
                                      "exec"] + argv, capture_output=True, text=True, timeout=45)
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
        print("PASS direct exec user/group/cwd/rlimit/environment/capabilities/privilege policy", flush=True)
        overridden = direct(["--env", "RUNE_EXEC_TEST=first", "--env", "RUNE_EXEC_TEST=last",
                             "--cwd", "/", cid, "/bin/sh", "-c", 'pwd; printf "%s\\n" "$RUNE_EXEC_TEST"'])
        assert overridden.splitlines() == ["/", "last"], "explicit overrides changed"
        process = root / "process.json"
        process.write_text(json.dumps({"user": {"uid": 1234, "gid": 1235}, "cwd": "/tmp",
                                       "env": ["PATH=/bin"], "args": ["/bin/sh", "-c", "ulimit -n; grep NoNewPrivs: /proc/self/status"]}))
        inherited = direct(["--process", str(process), cid]).splitlines()
        assert inherited[0] == "128" and inherited[1].split()[1] == "1", "process-file policy fallback changed"
        direct(["--process", str(process), "--cwd", "/", cid], expected=1)
        print("PASS explicit overrides, process-file inheritance and conflicting-option rejection", flush=True)
        after = json.loads(call(["inspect", cid]))[0]
        assert after["State"]["Pid"] == info["State"]["Pid"], "exec changed init identity"
    except Exception as error:
        errors.append(error)
    finally:
        if cid:
            try:
                call(["rm", "--force", cid])
            except Exception as error:
                errors.append(error)
        try:
            state = root / "state"
            if state.exists() and any(p.name != ".rune-locks" for p in state.iterdir()):
                errors.append(RuntimeError("runtime state remained after container cleanup"))
            call(["unshare", "python3", "-c", "import shutil,sys; shutil.rmtree(sys.argv[1])", str(root)])
        except Exception as error:
            errors.append(error)
    if errors:
        raise ExceptionGroup("exec policy validation or cleanup failed", errors)
    print("PASS init identity and disposable resource cleanup", flush=True)
    print("AppArmor denial requires an AppArmor-equipped test environment; no such claim here.")


if __name__ == "__main__":
    main()
