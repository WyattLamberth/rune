#!/usr/bin/env python3
"""Standalone input rejection; no kernel isolation or complete parity claim."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
from smoke_cleanup import run_bounded

ROOT = Path(__file__).resolve().parents[1]
REFERENCE_SHA = '599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488'


def sha256(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    reference = args.reference.resolve(strict=True)
    if sha256(reference) != REFERENCE_SHA:
        parser.error('reference differs from the authenticated pinned runc artifact')
    revision = git('rev-parse', 'HEAD')
    if git('status', '--porcelain', '--untracked-files=all'):
        parser.error('commit changes before exact-revision validation')
    subprocess.run(['bash', 'scripts/build-local.sh'], cwd=ROOT, check=True)
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1', '--no-deps'], cwd=ROOT))
    rune = Path(metadata['target_directory']) / 'debug/rune'
    report = {'rune_revision': revision, 'rune_binary_sha256': sha256(rune),
              'runc_revision': '29dd3dc2b13b4123162e5fe132504bb4b15569f1',
              'runc_binary_sha256': REFERENCE_SHA, 'cargo_lock_sha256': sha256(ROOT / 'Cargo.lock'),
              'suite_sha256': sha256(Path(__file__)),
              'helper_sha256': sha256(ROOT / 'scripts/smoke_cleanup.py'),
              'scope': __doc__, 'reference_source_build_reproduced': False,
              'deadline_seconds': 0.5, 'runtimes': {}, 'comparisons': {}}
    for kind, binary in [('runc', reference), ('rune', rune)]:
        cases = {}
        for case in ['writerless-fifo', 'directory', 'device-symlink', 'malformed-regular', 'malformed-regular-symlink']:
            with tempfile.TemporaryDirectory(prefix='rune-config-input-') as directory:
                root = Path(directory)
                bundle = root / 'bundle'
                bundle.mkdir()
                config = bundle / 'config.json'
                if case == 'writerless-fifo':
                    os.mkfifo(config, 0o600)
                elif case == 'directory':
                    config.mkdir()
                elif case == 'device-symlink':
                    config.symlink_to('/dev/null')
                elif case == 'malformed-regular-symlink':
                    (bundle / 'target.json').write_text('{}')
                    config.symlink_to('target.json')
                else:
                    config.write_text('{}')
                state = root / 'state'
                try:
                    result = run_bounded([str(binary), '--root', str(state), 'create', '--bundle', str(bundle), 'input-fixture'], timeout=0.5)
                    outcome = 'rejected' if result.returncode else 'unexpected-success'
                    rejected_type = b'OCI JSON input must be a regular file' in result.stderr
                except RuntimeError as error:
                    outcome = 'timeout' if 'timed out' in str(error) else 'runner-error'
                    rejected_type = False
                cases[case] = {'outcome': outcome, 'regular_file_rejection': rejected_type,
                               'container_records_created': (state / 'input-fixture').exists()}
        report['runtimes'][kind] = cases
    for case, right in report['runtimes']['rune'].items():
        left = report['runtimes']['runc'][case]
        report['comparisons'][case] = ('documented-policy-difference' if case == 'writerless-fifo' and left['outcome'] == 'timeout' and right['outcome'] == 'rejected'
                                      else 'matching-rejection' if left['outcome'] == right['outcome'] == 'rejected' else 'validation-gap')
    unchanged = git('rev-parse', 'HEAD') == revision and not git('status', '--porcelain', '--untracked-files=all')
    if not unchanged:
        report['rune_revision'] = None
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'rune_revision': report['rune_revision'], 'comparisons': report['comparisons']}))
    if not unchanged or any(case['container_records_created'] or case['outcome'] != 'rejected' for case in report['runtimes']['rune'].values()):
        raise SystemExit('candidate input validation failed; aggregate evidence retained')
    if any(not report['runtimes']['rune'][case]['regular_file_rejection'] for case in ['writerless-fifo', 'directory', 'device-symlink']):
        raise SystemExit('candidate did not reject nonregular input before parsing')
    if 'validation-gap' in report['comparisons'].values():
        raise SystemExit('reference input validation has an unexpected gap')


if __name__ == '__main__':
    main()
