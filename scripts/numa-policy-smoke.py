#!/usr/bin/env python3
"""Native NUMA init/exec policy checks with disposable rootless OCI resources."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
from smoke_cleanup import finish_resources, run_bounded, public_error
import tempfile
import uuid

IMAGE = 'docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15'
REFERENCE_SHA = '599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488'


def main():
    if sys.flags.optimize:
        raise SystemExit('validation requires Python assertions enabled')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--runtime-kind', choices=['rune', 'runc'], default='rune')
    parser.add_argument('--expect-exec', choices=['retained', 'missing'], default='retained')
    parser.add_argument('--results', type=Path)
    parser.add_argument('--preflight', action='store_true', help='also test safe invalid policies and rollback')
    args = parser.parse_args()
    runtime = args.runtime.resolve(strict=True)
    binary_sha = hashlib.sha256(runtime.read_bytes()).hexdigest()
    if args.runtime_kind == 'runc' and binary_sha != REFERENCE_SHA:
        raise SystemExit('reference is not the authenticated pinned runc artifact')
    root = Path(tempfile.mkdtemp(prefix='rune-numa-policy-'))
    # Podman prepares its ordinary rootless OCI bundle. The private fixture
    # adapter adds only the tested field before the native runtime reads it;
    # it forwards every invocation to that runtime and never executes workloads.
    policy_file = root / 'policy.json'
    policy_file.write_text(json.dumps({'mode': 'MPOL_LOCAL'}))
    wrapper = root / 'runtime'
    wrapper.write_text('#!/usr/bin/python3\nimport json,sys,os,pathlib\n'
        'args=sys.argv[1:]\n'
        'if "create" in args:\n'
        ' i=args.index("--bundle") if "--bundle" in args else args.index("-b")\n'
        ' p=pathlib.Path(args[i+1])/"config.json"\n'
        ' spec=json.loads(p.read_text())\n'
        f' spec["linux"]["memoryPolicy"]=json.loads(pathlib.Path({str(policy_file)!r}).read_text())\n'
        ' p.write_text(json.dumps(spec))\n'
        f'os.execv({str(runtime)!r}, [{str(runtime)!r}]+args)\n')
    wrapper.chmod(0o700)
    base = ['podman', '--root', str(root / 'store'), '--runroot', str(root / 'run'),
            '--storage-driver', 'vfs', '--runtime', str(wrapper),
            '--runtime-flag', f'root={root}/state']
    errors = []
    cid = None
    cases = {case: 'not_run' for case in ['init-policy', 'direct-exec-policy', 'process-file-policy', 'cleanup']}
    current = 'invalid-policy-rejection' if args.preflight else 'init-policy'
    if args.preflight:
        cases[current] = 'not_run'
    expected = 0 if args.expect_exec == 'retained' else 1

    def call(argv, timeout=45):
        result = run_bounded(base + argv, text=True, timeout=timeout)
        if result.returncode:
            raise RuntimeError(f'{argv[0]} exited {result.returncode}; output withheld')
        return result.stdout.strip()

    def direct(argv):
        result = run_bounded(base + ['unshare', str(runtime), '--root', str(root / 'state'),
                                       'exec'] + argv, timeout=45)
        assert result.returncode == expected, 'NUMA exec policy differs from expected classification'

    try:
        call(['pull', IMAGE], timeout=180)
        if args.preflight:
            for policy in [{'mode': 'MPOL_LOCAL', 'nodes': '1'},
                           {'mode': 'MPOL_BIND'}, {'mode': 'MPOL_BIND', 'nodes': '1024'}]:
                policy_file.write_text(json.dumps(policy))
                cid = call(['create', '--name', f'rune-numa-invalid-{uuid.uuid4().hex}',
                            '--network', 'none', IMAGE, 'true'])
                failed = run_bounded(base + ['start', cid], timeout=45)
                assert failed.returncode != 0, 'invalid NUMA policy unexpectedly started'
                state = root / 'state'
                assert not state.exists() or not any(p.name != '.rune-locks' for p in state.iterdir()), 'invalid-policy rollback left state'
                call(['rm', '--force', cid])
                cid = None
            cases[current] = 'pass'
            current = 'init-policy'
            policy_file.write_text(json.dumps({'mode': 'MPOL_LOCAL'}))
        cid = call(['create', '--name', f'rune-numa-{uuid.uuid4().hex}', '--network', 'none',
                    '--security-opt', 'no-new-privileges', IMAGE, 'sh', '-c',
                    'grep -q " local " /proc/self/numa_maps && exec sleep 120'])
        call(['start', cid])
        assert json.loads(call(['inspect', cid]))[0]['State']['Running'], 'init policy probe failed'
        # Check mappings of the configured init, independent of exec policy.
        call(['exec', cid, 'sh', '-c', 'grep -q " local " /proc/1/numa_maps'])
        cases[current] = 'pass'
        current = 'direct-exec-policy'
        direct([cid, 'sh', '-c', 'grep -q " local " /proc/self/numa_maps'])
        cases[current] = 'pass'
        current = 'process-file-policy'
        process = root / 'process.json'
        process.write_text(json.dumps({'user': {'uid': 0, 'gid': 0}, 'cwd': '/',
            'args': ['/bin/sh', '-c', 'grep -q " local " /proc/self/numa_maps'],
            'env': ['PATH=/bin:/usr/bin'], 'noNewPrivileges': True}))
        direct(['--process', str(process), cid])
        cases[current] = 'pass'
    except Exception as error:
        cases[current] = 'validation_gap'
        errors.append(public_error(error))
    finally:
        before = len(errors)
        cleanup_report = finish_resources(root, runtime, call, errors,
            Path(__file__).resolve().parents[1] / 'work/smoke-recovery')
        cases['cleanup'] = 'pass' if len(errors) == before else 'validation_gap'
    results = {'runtime_kind': args.runtime_kind, 'binary_sha256': binary_sha,
        'image': IMAGE, 'policy': 'MPOL_LOCAL', 'expected_exec_policy': args.expect_exec,
        'cases': cases, 'cleanup': cleanup_report, 'gaps': ['other-NUMA-modes-and-flags', 'malformed-policy-preflight',
            'denied-kernel-features', 'rootful', 'architectures', 'concurrency']}
    if args.results:
        args.results.write_text(json.dumps(results, indent=2) + '\n')
    print(json.dumps({'runtime_kind': args.runtime_kind, 'cases': cases}), flush=True)
    if errors:
        raise ExceptionGroup('NUMA validation or cleanup failed; unavailable configurations remain gaps', errors)


if __name__ == '__main__':
    main()
