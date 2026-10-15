#!/usr/bin/env python3
"""Standalone delete CLI/filesystem cases; no kernel container isolation claim."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import uuid
sys.dont_write_bytecode = True
from smoke_cleanup import run_bounded

ROOT = Path(__file__).resolve().parents[1]
REFERENCE_SHA = '599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--allow-dirty', action='store_true')
    args = parser.parse_args()
    reference = args.reference.resolve(strict=True)
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    if digest(reference) != REFERENCE_SHA:
        parser.error('reference differs from pinned signed artifact')
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    dirty = subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=ROOT, text=True)
    if dirty and not args.allow_dirty:
        parser.error('commit before exact validation')
    subprocess.run(['bash', 'scripts/build-local.sh'], cwd=ROOT, check=True)
    cargo = json.loads(subprocess.check_output(['cargo', 'metadata', '--format-version', '1', '--no-deps', '--locked'], cwd=ROOT))
    native = Path(cargo['target_directory']) / 'debug/rune'
    report = {'rune_revision': None if dirty else head, 'development_base': head if dirty else None,
        'runc_revision': '29dd3dc2b13b4123162e5fe132504bb4b15569f1', 'runc_binary_sha256': REFERENCE_SHA,
        'rune_binary_sha256': digest(native), 'cargo_lock_sha256': digest(ROOT / 'Cargo.lock'),
        'suite_sha256': digest(Path(__file__)), 'runtimes': {}, 'comparisons': {},
        'scope': 'Production CLI/filesystem absence and partial-ownership contract; not kernel isolation'}
    for kind, binary in [('runc', reference), ('rune', native)]:
        with tempfile.TemporaryDirectory(prefix='rune-delete-absence-') as directory:
            root = Path(directory) / 'state'
            root.mkdir(mode=0o700)
            identifier = 'owned-' + uuid.uuid4().hex
            def call(force):
                return run_bounded([str(binary), '--root', str(root), 'delete'] + (['--force'] if force else []) + [identifier], timeout=2)
            first, second, ordinary = call(True), call(True), call(False)
            partial = root / identifier
            partial.mkdir()
            marker = partial / 'ownership'
            marker.write_bytes(b'synthetic-owned-record')
            partial_result = call(True)
            report['runtimes'][kind] = {'force_absent_codes': [first.returncode, second.returncode],
                'ordinary_absent_code': ordinary.returncode, 'partial_code': partial_result.returncode,
                'partial_retained': marker.exists(), 'force_stdout_empty': not first.stdout and not second.stdout}
    left, right = report['runtimes']['runc'], report['runtimes']['rune']
    matched = all(value['force_absent_codes'] == [0, 0] and value['ordinary_absent_code'] != 0
                  and value['force_stdout_empty'] for value in [left, right])
    partial_difference = left['partial_code'] == 0 and not left['partial_retained'] and right['partial_code'] != 0 and right['partial_retained']
    report['comparisons'] = {'complete-absence': 'matching_assertions' if matched else 'validation_gap',
                            'partial-ownership': 'known_difference' if partial_difference else 'validation_gap'}
    after = subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=ROOT, text=True)
    changed = after != dirty or subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip() != head
    if changed:
        report['rune_revision'] = None
        report['source_changed_during_validation'] = True
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'rune_revision': report['rune_revision'], 'comparisons': report['comparisons']}))
    if changed or not matched or not partial_difference:
        raise SystemExit('delete contract validation failed; aggregate gaps retained')


if __name__ == '__main__':
    main()
