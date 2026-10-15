#!/usr/bin/env python3
"""Rootless Podman checks, using isolated disposable storage and real isolation."""
import argparse
import json
from pathlib import Path
import tempfile
import sys
sys.dont_write_bytecode = True
from smoke_cleanup import finish_resources, state_summary, run_bounded, public_error
from smoke_scope import make_runtime, summary

IMAGE = 'docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15'
# The image can be explicitly overridden for offline testing. The validated
# default digest is filled from a registry pull, not guessed from a tag.

def main():
    if sys.flags.optimize:
        raise SystemExit("validation requires Python assertions enabled")
    parser = argparse.ArgumentParser()
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--image', default=IMAGE)
    parser.add_argument('--results', type=Path)
    parser.add_argument('--runtime-kind', choices=['rune', 'runc'], default='rune')
    args = parser.parse_args()
    runtime = args.runtime.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix='rune-smoke-'))
    runtime, scope_trace = make_runtime(runtime, root / "state")
    errors = []
    phase = "prepare"
    selinux_available = Path('/sys/fs/selinux/enforce').exists()
    selinux_checks = {case: 'not-run' if selinux_available else 'unavailable' for case in ['init', 'exec']}
    try:
        base = ['podman', '--root', str(root/'store'), '--runroot', str(root/'run'),
                '--storage-driver', 'vfs', '--runtime', str(runtime),
                '--runtime-flag', f'root={root}/state']
        def call(argv, code=0, timeout=45):
            result = run_bounded(base+argv, text=True, timeout=timeout)
            if result.returncode != code:
                raise RuntimeError(f'{argv[0]} exited {result.returncode}, expected {code}; output withheld')
            return result.stdout.strip()
        call(['pull', args.image], timeout=180)
        phase = 'output-and-exit'
        common = ['run', '--rm', '--network', 'none']
        assert call(common+[args.image, 'echo', 'rune-ok']) == 'rune-ok'
        call(common+[args.image, 'sh', '-c', 'exit 17'], 17)
        print('PASS output and exit status', flush=True)
        phase = 'resource-limits'
        limits = call(common+['--memory', '64m', '--pids-limit', '32', args.image,
                       'sh', '-c', 'cat /proc/self/cgroup /sys/fs/cgroup/memory.max /sys/fs/cgroup/pids.max'])
        assert limits.splitlines() == ['0::/', '67108864', '32'], 'resource policy mismatch'
        print('PASS cgroup namespace and memory/PID limits', flush=True)
        phase = 'security'
        caps = call(common+['--cap-drop', 'all', args.image, 'sh', '-c', 'grep CapEff: /proc/self/status'])
        assert int(caps.split()[1], 16) == 0, 'capabilities mismatch'
        nnp = call(common+['--security-opt', 'no-new-privileges', args.image, 'sh', '-c', 'grep NoNewPrivs: /proc/self/status'])
        assert int(nnp.split()[1]) == 1, 'privilege policy mismatch'
        init_seccomp = call(common+[args.image, 'sh', '-c', 'grep Seccomp: /proc/self/status'])
        assert int(init_seccomp.split()[1]) == 2, 'init seccomp policy mismatch'
        if selinux_available:
            selinux_checks['init'] = 'fail'
            init_label = call(common+[args.image, 'cat', '/proc/self/attr/current']).rstrip('\0')
            assert ':container_t:' in init_label, 'init SELinux domain mismatch'
            selinux_checks['init'] = 'pass'
        print('PASS dropped capabilities and no-new-privileges', flush=True)
        print('PASS init seccomp and conditional SELinux domain', flush=True)
        phase = 'lifecycle'
        cid = call(['create', '--network', 'none', args.image, 'sleep', '120'])
        try:
            call(['start', cid])
            info = json.loads(call(['inspect', cid]))[0]
            assert Path(info['OCIRuntime']).resolve() == runtime
            assert info['State']['Status'] == 'running'
            output = call(['exec', cid, 'sh', '-c', 'grep Seccomp: /proc/self/status; cat /proc/self/attr/current 2>/dev/null || true'])
            assert int(output.splitlines()[0].split()[1]) == 2, 'seccomp policy mismatch'
            if selinux_available:
                selinux_checks['exec'] = 'fail'
                expected = info['ProcessLabel']
                actual = output.splitlines()[1].rstrip('\0')
                assert expected and actual == expected, 'SELinux policy mismatch'
                assert ':container_t:' in actual, 'SELinux domain mismatch'
                selinux_checks['exec'] = 'pass'
            call(['exec', cid, 'sh', '-c', 'exit 23'], 23)
            # Exec must not replace the container init identity or state.
            after = json.loads(call(['inspect', cid]))[0]
            assert after['State']['Pid'] == info['State']['Pid']
            call(['pause', cid]); call(['unpause', cid])
            call(['kill', '--signal', 'KILL', cid])
            assert call(['wait', cid]) == '137'
            call(['rm', cid])
            cid = None
        finally:
            if cid:
                try:
                    call(['rm', '--force', cid])
                except Exception as error:
                    errors.append(public_error(error))
        print('PASS exec security, exec exit status, lifecycle and init identity', flush=True)
        phase = 'tty'
        assert call(common+['--tty', args.image, 'sh', '-c', 'test -t 1 && echo tty-ok']) == 'tty-ok'
        print('PASS TTY', flush=True)
        phase = 'network'
        # A normal Podman invocation creates the network namespace via pasta.
        assert call(['run', '--rm', args.image, 'sh', '-c', 'test "$(ls /sys/class/net | grep -v ^lo$ | wc -l)" -ge 1 && echo network-ok']) == 'network-ok'
        print('PASS default Podman network namespace', flush=True)
        phase = 'cleanup-check'
        assert not call(['ps', '--all', '--format', '{{.ID}}'])
        assert state_summary(root)['entries'] == 0, 'runtime state remained after cleanup'
        print('PASS container and runtime state cleanup', flush=True)
        phase = 'completed'
    except Exception as error:
        errors.append(public_error(error))
    finally:
        # Remove only containers in this generated private store. Preserve owned
        # state and a private recovery locator when cleanup cannot be confirmed.
        report = finish_resources(root, runtime, call, errors,
                                  Path(__file__).resolve().parents[1] / 'work/smoke-recovery')
    if args.results:
        args.results.write_text(json.dumps({'runtime_kind': args.runtime_kind,
            'cases': {'general-kernel': 'pass' if not errors else 'validation_gap'},
            'cleanup': report, 'runtime_scope': summary(scope_trace), 'phase': phase, 'error_types': [type(error).__name__ for error in errors],
            'selinux_domain_checks': selinux_checks, 'gaps': ['complete-conformance', 'kernel-leak-enumeration',
                'interrupted-acquisition', 'concurrency']}, indent=2) + '\n')
    if errors:
        raise ExceptionGroup('general smoke validation or cleanup failed; output withheld', errors)
    print('All rootless Podman smoke checks passed.')

if __name__ == '__main__':
    main()
