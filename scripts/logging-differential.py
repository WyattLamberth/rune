#!/usr/bin/env python3
"""Compare diagnostic framing with pinned runc; no kernel isolation claim."""
import argparse
from datetime import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from smoke_cleanup import run_bounded

ROOT = Path(__file__).resolve().parents[1]
REFERENCE_SHA = '599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    reference = args.reference.resolve(strict=True)
    if digest(reference) != REFERENCE_SHA:
        parser.error('reference differs from pinned signed artifact')
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    status = lambda: subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=ROOT, text=True)
    if status():
        parser.error('commit before exact validation')
    subprocess.run(['bash', 'scripts/build-local.sh'], cwd=ROOT, check=True)
    cargo = json.loads(subprocess.check_output(['cargo', 'metadata', '--format-version', '1', '--no-deps', '--locked'], cwd=ROOT))
    native = Path(cargo['target_directory']) / 'debug/rune'
    report = {'rune_revision': head, 'runc_revision': '29dd3dc2b13b4123162e5fe132504bb4b15569f1',
              'rune_binary_sha256': digest(native), 'runc_binary_sha256': REFERENCE_SHA,
              'cargo_lock_sha256': digest(ROOT / 'Cargo.lock'), 'suite_sha256': digest(Path(__file__)),
              'scope': 'Standalone CLI diagnostic framing, append and error paths; not full logging parity, kernel isolation or cross-process ordering',
              'runtimes': {}, 'matching_assertions': False}
    for kind, binary in [('runc', reference), ('rune', native)]:
        results = {}
        for format in ['text', 'json']:
            for debug in [False, True]:
                for destination in ['file', 'stderr']:
                    with tempfile.TemporaryDirectory(prefix='rune-logging-') as directory:
                        root = Path(directory) / 'state'
                        root.mkdir(mode=0o700)
                        log = Path(directory) / 'diagnostics'
                        prefix = 'previous-record\n'
                        log.write_text(prefix)
                        command = [str(binary), '--root', str(root), '--log-format', format]
                        if debug:
                            command += ['--debug']
                        if destination == 'file':
                            command += ['--log', str(log)]
                        result = run_bounded(command + ['state', 'missing'], timeout=5)
                        text = log.read_text() if destination == 'file' else result.stderr
                        append_ok = destination != 'file' or text.startswith(prefix)
                        if destination == 'file':
                            text = text[len(prefix):]
                        lines = text.splitlines()
                        schema_ok = bool(lines)
                        if format == 'json':
                            try:
                                for line in lines:
                                    record = json.loads(line)
                                    schema_ok &= record['level'] == 'error' and isinstance(record['msg'], str)
                                    timestamp = datetime.fromisoformat(record['time'].replace('Z', '+00:00'))
                                    schema_ok &= timestamp.tzinfo is not None
                            except (ValueError, KeyError, TypeError):
                                schema_ok = False
                        results[f'{format}-{debug}-{destination}'] = {
                            'nonzero': result.returncode != 0, 'stdout_empty': not result.stdout,
                            'append_preserved': append_ok, 'diagnostics_framed': schema_ok,
                            'stderr_present': bool(result.stderr)}
        report['runtimes'][kind] = results
    report['matching_assertions'] = all(all(case.values()) for cases in report['runtimes'].values() for case in cases.values())
    changed = bool(status()) or subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip() != head
    if changed:
        report['rune_revision'] = None
        report['source_changed_during_validation'] = True
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'rune_revision': report['rune_revision'], 'matching_assertions': report['matching_assertions']}))
    if changed or not report['matching_assertions']:
        raise SystemExit('diagnostic comparison failed; aggregate gaps retained')


if __name__ == '__main__':
    main()
