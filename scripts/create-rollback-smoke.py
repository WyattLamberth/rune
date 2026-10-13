#!/usr/bin/env python3
"""Failed native create checks using private disposable rootless Podman storage."""
import argparse
from pathlib import Path
import subprocess
import sys
import tempfile
import uuid

IMAGE = 'docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15'


def main():
    if sys.flags.optimize:
        raise SystemExit('validation requires Python assertions enabled')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    args = parser.parse_args()
    runtime = args.runtime.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix='rune-create-rollback-'))
    base = ['podman', '--root', str(root / 'store'), '--runroot', str(root / 'run'),
            '--storage-driver', 'vfs', '--runtime', str(runtime),
            '--runtime-flag', f'root={root}/state']
    errors = []
    names = []

    def call(argv, timeout=45):
        result = subprocess.run(base + argv, capture_output=True, text=True, timeout=timeout)
        if result.returncode:
            raise RuntimeError(f'{argv[0]} exited {result.returncode}; output withheld')
        return result.stdout.strip()

    try:
        call(['pull', IMAGE], timeout=180)
        for case, command in [('missing-executable', '/rune-no-such-executable'),
                              ('non-executable', '/etc/alpine-release')]:
            name = f'rune-rollback-{uuid.uuid4().hex}'
            names.append(name)
            call(['create', '--name', name, '--network', 'none', IMAGE, command])
            failed = subprocess.run(base + ['start', name], capture_output=True, timeout=45)
            assert failed.returncode != 0, 'invalid executable unexpectedly started'
            # Inspect before Podman rm can mask a failed runtime rollback.
            state = root / 'state'
            assert state.is_dir(), 'native runtime did not create its state root'
            assert not any(p.name != '.rune-locks' for p in state.iterdir()), 'failed create retained runtime state'
            call(['rm', '--force', name])
            names.remove(name)
            print(f'PASS failed create {case} and runtime state rollback', flush=True)
        assert not call(['ps', '--all', '--format', '{{.ID}}']), 'Podman containers remained'
    except Exception as error:
        errors.append(error)
    finally:
        for name in names:
            try:
                call(['rm', '--force', name])
            except Exception as error:
                errors.append(error)
        try:
            call(['unshare', 'python3', '-c', 'import shutil,sys; shutil.rmtree(sys.argv[1])', str(root)])
        except Exception as error:
            errors.append(error)
    if errors:
        raise ExceptionGroup('create rollback validation or cleanup failed', errors)
    print('PASS disposable resource cleanup')
    print('Kernel cgroup/RDT failure injection, leak enumeration and concurrency remain gaps.')


if __name__ == '__main__':
    main()
