#!/usr/bin/env python3
"""Rootless Podman checks, using isolated disposable storage and real isolation."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile
import shutil

IMAGE = 'docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15'
# The image can be explicitly overridden for offline testing. The validated
# default digest is filled from a registry pull, not guessed from a tag.

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--image', default=IMAGE)
    args = parser.parse_args()
    runtime = args.runtime.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix='rune-smoke-'))
    try:
        base = ['podman', '--root', str(root/'store'), '--runroot', str(root/'run'),
                '--storage-driver', 'vfs', '--runtime', str(runtime),
                '--runtime-flag', f'root={root}/state']
        def call(argv, code=0, timeout=45):
            result = subprocess.run(base+argv, text=True, capture_output=True, timeout=timeout)
            if result.returncode != code:
                raise RuntimeError(f'{argv[0]} failed ({result.returncode}, expected {code}): {result.stderr}')
            return result.stdout.strip()
        call(['pull', args.image], timeout=180)
        common = ['run', '--rm', '--network', 'none']
        assert call(common+[args.image, 'echo', 'rune-ok']) == 'rune-ok'
        call(common+[args.image, 'sh', '-c', 'exit 17'], 17)
        print('PASS output and exit status', flush=True)
        limits = call(common+['--memory', '64m', '--pids-limit', '32', args.image,
                       'sh', '-c', 'cat /proc/self/cgroup /sys/fs/cgroup/memory.max /sys/fs/cgroup/pids.max'])
        assert limits.splitlines() == ['0::/', '67108864', '32'], limits
        print('PASS cgroup namespace and memory/PID limits', flush=True)
        caps = call(common+['--cap-drop', 'all', args.image, 'sh', '-c', 'grep CapEff: /proc/self/status'])
        assert int(caps.split()[1], 16) == 0, caps
        nnp = call(common+['--security-opt', 'no-new-privileges', args.image, 'sh', '-c', 'grep NoNewPrivs: /proc/self/status'])
        assert int(nnp.split()[1]) == 1, nnp
        print('PASS dropped capabilities and no-new-privileges', flush=True)
        cid = call(['create', '--network', 'none', args.image, 'sleep', '120'])
        try:
            call(['start', cid])
            info = json.loads(call(['inspect', cid]))[0]
            assert Path(info['OCIRuntime']).resolve() == runtime
            assert info['State']['Status'] == 'running'
            output = call(['exec', cid, 'sh', '-c', 'grep Seccomp: /proc/self/status; cat /proc/self/attr/current 2>/dev/null || true'])
            assert int(output.splitlines()[0].split()[1]) == 2, output
            if Path('/sys/fs/selinux/enforce').exists():
                expected = info['ProcessLabel']
                actual = output.splitlines()[1].rstrip('\0')
                assert expected and actual == expected, (expected, actual)
                assert ':container_t:' in actual, actual
            call(['exec', cid, 'sh', '-c', 'exit 23'], 23)
            # Exec must not replace the container init identity or state.
            after = json.loads(call(['inspect', cid]))[0]
            assert after['State']['Pid'] == info['State']['Pid']
            call(['pause', cid]); call(['unpause', cid])
            call(['kill', '--signal', 'KILL', cid])
            assert call(['wait', cid]) == '137'
            call(['rm', cid])
        finally:
            subprocess.run(base+['rm', '-f', cid], capture_output=True, timeout=45)
        print('PASS exec security, exec exit status, lifecycle and init identity', flush=True)
        assert call(common+['--tty', args.image, 'sh', '-c', 'test -t 1 && echo tty-ok']) == 'tty-ok'
        print('PASS TTY', flush=True)
        # A normal Podman invocation creates the network namespace via pasta.
        assert call(['run', '--rm', args.image, 'sh', '-c', 'test "$(ls /sys/class/net | grep -v ^lo$ | wc -l)" -ge 1 && echo network-ok']) == 'network-ok'
        print('PASS default Podman network namespace', flush=True)
        assert not call(['ps', '--all', '--format', '{{.ID}}'])
        state = root/'state'
        assert not [item for item in state.iterdir() if item.name != '.rune-locks']
        print('PASS container and runtime state cleanup', flush=True)
    finally:
        # VFS layers can be owned by subordinate UIDs; clean only this generated
        # temporary tree from Podman's own user namespace.
        cleanup = subprocess.run(base+['unshare', 'python3', '-c', 'import shutil,sys; shutil.rmtree(sys.argv[1])', str(root)], capture_output=True, timeout=45)
        if cleanup.returncode:
            shutil.rmtree(root, ignore_errors=True)
            if root.exists():
                raise RuntimeError(f'Cannot clean disposable smoke directory: {root}')
    print('All rootless Podman smoke checks passed.')

if __name__ == '__main__':
    main()
