#!/usr/bin/env python3
"""Pinned two-hook comparison with explicit OCI/reference conflict classification."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from smoke_cleanup import run_bounded

ROOT=Path(__file__).resolve().parents[1]
REF_SHA='599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488'


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--allow-dirty',action='store_true')
    args=parser.parse_args()
    sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
    reference=args.reference.resolve(strict=True)
    if sha(reference)!=REF_SHA:parser.error('reference differs from pinned signed artifact')
    git=lambda argv:subprocess.check_output(['git']+argv,cwd=ROOT,text=True).strip()
    head=git(['rev-parse','HEAD']);dirty=git(['status','--porcelain','--untracked-files=all'])
    if dirty and not args.allow_dirty:parser.error('commit before exact validation')
    subprocess.run(['bash','scripts/build-local.sh'],cwd=ROOT,check=True)
    cargo=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--no-deps','--locked'],cwd=ROOT))
    native=Path(cargo['target_directory'])/'debug/rune'
    report={'rune_revision':None if dirty else head,'development_base':head if dirty else None,
            'runc_revision':'29dd3dc2b13b4123162e5fe132504bb4b15569f1','runc_binary_sha256':REF_SHA,
            'rune_binary_sha256':sha(native),'cargo_lock_sha256':sha(ROOT/'Cargo.lock'),
            'fixtures_sha256':{name:sha(ROOT/'scripts'/name) for name in ['poststop-probe.py','smoke_scope.py','smoke_cleanup.py']},
            'runtimes':{},'comparisons':{},'scope':'Rootless two-hook failure/order; OCI/reference conflict remains open'}
    with tempfile.TemporaryDirectory(prefix='rune-poststop-results-') as directory:
        for kind,binary in [('runc',reference),('rune',native)]:
            output=Path(directory)/(kind+'.json')
            result=run_bounded([sys.executable,str(ROOT/'scripts/poststop-probe.py'),'--runtime',str(binary),
                                '--kind',kind,'--results',str(output)],timeout=300)
            report['runtimes'][kind]={'exit_code':result.returncode,'results':json.loads(output.read_text()) if output.exists() else None}
    left=report['runtimes']['runc']['results'];right=report['runtimes']['rune']['results']
    if left and right:
        deleted=lambda value:not value.get('state_directory_retained',True) and bool(value['observations']) and all(not event['state_directory_present'] for event in value['observations'])
        report['comparisons']={'deletion-order':'matching_assertions' if deleted(left) and deleted(right) else 'validation_gap',
            'failure-exit':'matching_reference_with_OCI_conflict' if left.get('delete_exit_code')==right.get('delete_exit_code')==1 else 'validation_gap',
            'remaining-hooks':'reference_spec_conflict' if not left.get('remaining_hook_invoked') and not right.get('remaining_hook_invoked') else 'validation_gap',
            'warning-contract':'reference_spec_conflict' if not left.get('oci_contract_satisfied') and not right.get('oci_contract_satisfied') else 'validation_gap'}
    changed=git(['rev-parse','HEAD'])!=head or git(['status','--porcelain','--untracked-files=all'])!=dirty
    if changed:report['rune_revision']=None;report['source_changed_during_validation']=True
    args.output.write_text(json.dumps(report,indent=2,sort_keys=True)+'\n')
    print(json.dumps({'rune_revision':report['rune_revision'],'comparisons':report['comparisons']}))
    if changed or any(value['exit_code'] for value in report['runtimes'].values()):
        raise SystemExit('poststop investigation failed; aggregate gaps retained')


if __name__=='__main__':main()
