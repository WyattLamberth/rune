#!/usr/bin/env python3
"""Live native deletion faults and retry, using private rootless Podman resources."""
import argparse
import json
from pathlib import Path
import re
import sys
import tempfile
import uuid
sys.dont_write_bytecode = True
from smoke_cleanup import finish_resources, run_bounded, public_error
from smoke_scope import make_runtime, summary

IMAGE = 'docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15'


def main():
    if sys.flags.optimize:
        raise SystemExit('validation requires Python assertions enabled')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--results', type=Path)
    args = parser.parse_args()
    native = args.runtime.resolve(strict=True)
    root = Path(tempfile.mkdtemp(prefix='rune-cleanup-metadata-'))
    runtime, trace = make_runtime(native, root / 'state')
    base = ['podman', '--root', str(root / 'store'), '--runroot', str(root / 'run'),
            '--storage-driver', 'vfs', '--runtime', str(runtime), '--runtime-flag', f'root={root}/state']
    errors = []
    saved = False
    directory = None
    cases = {case: 'not_run' for case in ['missing', 'malformed', 'fifo', 'retry', 'cleanup']}
    current = 'missing'

    def call(argv, timeout=45):
        result = run_bounded(base + argv, timeout=timeout, text=True)
        if result.returncode:
            raise RuntimeError(f'owned {argv[0]} failed; output withheld')
        return result.stdout.strip()

    def direct(argv):
        return run_bounded(base + ['unshare', str(runtime), '--root', str(root / 'state')] + argv,
                           timeout=15, text=True)

    def mutate(mode):
        # All paths derive from this generated private root and validated full ID.
        code = ('import pathlib,os,sys,hashlib; p=pathlib.Path(sys.argv[1]); m=p/"youki_config.json";'
                ' b=p/"saved-cleanup.json"; mode=sys.argv[2]; '
                '\nif mode=="save":\n m.rename(b); (p/"state-check.sha").write_text(hashlib.sha256((p/"state.json").read_bytes()).hexdigest())'
                '\nelif mode=="malformed": m.write_bytes(b"not-json")'
                '\nelif mode=="fifo":\n m.unlink(); os.mkfifo(m,0o600)'
                '\nelif mode=="check":\n assert hashlib.sha256((p/"state.json").read_bytes()).hexdigest()==(p/"state-check.sha").read_text()'
                '\nelif mode=="restore":\n'
                ' if b.exists():\n  if m.exists(): m.unlink()\n  b.rename(m)')
        call(['unshare', 'python3', '-c', code, str(directory), mode])

    try:
        call(['pull', IMAGE], timeout=180)
        cid = call(['create', '--name', f'rune-cleanup-{uuid.uuid4().hex}', '--network', 'none',
                    '--cap-drop', 'all', '--security-opt', 'no-new-privileges', IMAGE, 'sleep', '120'])
        if not re.fullmatch('[0-9a-f]{64}', cid):
            raise RuntimeError('unexpected owned container ID')
        directory = root / 'state' / cid
        call(['start', cid])
        initial = direct(['state', cid])
        assert initial.returncode == 0
        initial = json.loads(initial.stdout)
        assert initial['status'] == 'running'
        saved = True; mutate('save')
        for mode in ['missing', 'malformed', 'fifo']:
            current = mode
            if mode != 'missing':
                mutate(mode)
            result = direct(['delete', '--force', cid])
            assert result.returncode != 0, 'unusable cleanup metadata reported success'
            assert 'config' in result.stderr.lower(), 'fault did not reach cleanup metadata validation'
            mutate('check')
            state = direct(['state', cid])
            assert state.returncode == 0
            state = json.loads(state.stdout)
            assert state['status'] == 'running' and state['pid'] == initial['pid'], 'owned init or state changed'
            cases[mode] = 'pass'
        mutate('restore'); saved = False
        current = 'retry'
        result = direct(['delete', '--force', cid])
        assert result.returncode == 0, 'cleanup did not recover after restoring metadata'
        assert not directory.exists(), 'successful delete retained ownership records'
        cases[current] = 'pass'
    except Exception as error:
        cases[current] = 'validation_gap'
        errors.append(public_error(error))
    finally:
        if saved:
            try:
                mutate('restore')
            except Exception as error:
                errors.append(public_error(error))
        before = len(errors)
        cleanup = finish_resources(root, runtime, call, errors, Path(__file__).resolve().parents[1] / 'work/smoke-recovery')
        cases['cleanup'] = 'pass' if len(errors) == before else 'validation_gap'
    if args.results:
        args.results.write_text(json.dumps({'cases': cases, 'cleanup': cleanup, 'runtime_scope': summary(trace),
            'scope': 'Live rootless rune cleanup-metadata rejection and retry; not runc parity or exhaustive recovery',
            'gaps': ['cgroup-failure', 'hook-failure', 'interrupted-acquisition', 'concurrency', 'rootful', 'architectures']}, indent=2) + '\n')
    print(json.dumps({'cases': cases}))
    if errors:
        raise ExceptionGroup('cleanup metadata validation or owned cleanup failed', errors)


if __name__ == '__main__':
    main()
