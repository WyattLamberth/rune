#!/usr/bin/env python3
"""Two-hook native kernel investigation; preserve reference/spec discrepancies."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import tempfile
import time
import uuid
sys.dont_write_bytecode = True
from smoke_cleanup import finish_resources, run_bounded, public_error
from smoke_scope import make_runtime, summary

IMAGE = 'docker.io/library/alpine@sha256:1f3591b8a02ea153f41c5bba878ad477f63ab3d19349762cb77504db02a23e15'
REFERENCE_SHA = '599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--kind', choices=['rune', 'runc'], required=True)
    parser.add_argument('--results', type=Path, required=True)
    parser.add_argument("--scenario", choices=["failure", "backpressure", "eof"], default="failure")
    args = parser.parse_args()
    native = args.runtime.resolve(strict=True)
    digest = hashlib.sha256(native.read_bytes()).hexdigest()
    if args.kind == 'runc' and digest != REFERENCE_SHA:
        parser.error('reference differs from pinned signed artifact')
    root = Path(tempfile.mkdtemp(prefix='rune-poststop-probe-'))
    scoped, trace = make_runtime(native, root / 'state')
    hook = root / 'hook.py'
    observations = root / 'hooks.jsonl'
    hook.write_text('import json,os,pathlib,sys\n'
        'value=json.loads(sys.stdin.read()); slot=int(sys.argv[1]); state=pathlib.Path(sys.argv[2])\n'
        'record={"slot":slot,"state_directory_present":state.exists(),'
        '"status":value.get("status") if value.get("status") in ["stopped","running","created"] else "unknown",'
        '"state_fields_present":all(k in value for k in ["id","bundle","ociVersion","status"])}\n'
        'fd=os.open(sys.argv[3],os.O_WRONLY|os.O_CREAT|os.O_APPEND|os.O_CLOEXEC,0o600)\n'
        'os.write(fd,(json.dumps(record)+"\\n").encode());os.close(fd)\n'
        'sys.exit(7 if slot==1 else 0)\n')
    pids = root / 'hook-pids.jsonl'
    if args.scenario != 'failure':
        hook.write_text('import json,os,pathlib,sys,time\n'
            'slot=int(sys.argv[1]);directory=pathlib.Path(sys.argv[2])\n'
            f'fd=os.open({str(pids)!r},os.O_WRONLY|os.O_CREAT|os.O_APPEND|os.O_CLOEXEC,0o600)\n'
            'os.write(fd,(str(os.getpid())+"\\n").encode());os.close(fd)\n'
            + ('time.sleep(4)\n' if args.scenario == 'backpressure' else
               'value=json.loads(sys.stdin.read())\n'
               'record={"slot":slot,"state_directory_present":directory.exists(),"status":value.get("status"),'
               '"state_fields_present":all(k in value for k in ["id","bundle","ociVersion","status"])}\n'
               'fd=os.open(sys.argv[3],os.O_WRONLY|os.O_CREAT|os.O_APPEND|os.O_CLOEXEC,0o600)\n'
               'os.write(fd,(json.dumps(record)+"\\n").encode());os.close(fd)\n'))
    fixture = root / native.name
    fixture.write_text('#!/usr/bin/python3\nimport json,os,pathlib,re,sys\nargs=sys.argv[1:]\n'
        'if "create" in args:\n'
        ' identifier=args[-1]\n assert re.fullmatch("[0-9a-f]{64}",identifier)\n'
        ' i=args.index("--bundle") if "--bundle" in args else args.index("-b")\n'
        ' p=pathlib.Path(args[i+1])/"config.json"\n spec=json.loads(p.read_text())\n'
        + (' spec["annotations"]={"rune.fixture":"x"*262144}\n' if args.scenario != 'failure' else '')+
        f' directory=str(pathlib.Path({str(root / "state")!r})/identifier)\n'
        ' spec["hooks"]={"poststop":[{"path":"/usr/bin/python3",'
        f'"args":["python3",{str(hook)!r},str(slot),directory,{str(observations)!r}],"timeout":1}} for slot in [1,2]]}}\n'
        ' p.write_text(json.dumps(spec))\n'
        f'os.execv({str(scoped)!r},[{str(scoped)!r}]+args)\n')
    fixture.chmod(0o700)
    base = ['podman','--root',str(root/'store'),'--runroot',str(root/'run'),'--storage-driver','vfs',
            '--runtime',str(fixture),'--runtime-flag',f'root={root}/state']
    errors = []
    cid = None
    report = {'kind': args.kind, 'binary_sha256': digest, 'image': IMAGE, 'observations': []}
    def call(argv, timeout=45):
        result = run_bounded(base+argv, timeout=timeout, text=True)
        if result.returncode:
            raise RuntimeError(f'owned {argv[0]} failed; output withheld')
        return result.stdout.strip()
    try:
        call(['pull',IMAGE],timeout=180)
        cid=call(['create','--name','rune-hook-'+uuid.uuid4().hex,'--network','none',IMAGE,'sleep','120'])
        if not re.fullmatch('[0-9a-f]{64}',cid):raise RuntimeError('unexpected owned ID')
        call(['start',cid])
        started=time.monotonic()
        result=run_bounded(base+['unshare',str(scoped),'--root',str(root/'state'),'delete','--force',cid],timeout=15,text=True)
        report['scenario']=args.scenario
        report['within_three_seconds']=time.monotonic()-started<3
        report['delete_exit_code']=result.returncode
        report['observed_hooks_absent']=True
        report['hook_process_observations']=0
        if pids.exists():
            payload=pids.read_bytes()
            if len(payload)>1024:raise RuntimeError('hook PID observation exceeded bound')
            owned=payload.decode().splitlines()
            if not all(re.fullmatch('[1-9][0-9]{0,9}',value) for value in owned):raise RuntimeError('invalid private hook PID')
            report['hook_process_observations']=len(owned)
            report['observed_hooks_absent']=all(not Path('/proc',value).exists() for value in owned)
        report['state_directory_retained']=(root/'state'/cid).exists()
        if observations.exists():
            payload=observations.read_bytes()
            if len(payload)>65536:raise RuntimeError('hook observation exceeded bound')
            report['observations']=[json.loads(line) for line in payload.splitlines()]
        report['remaining_hook_invoked']=any(value['slot']==2 for value in report['observations'])
        report['warning_observed']='warn' in result.stderr.lower()
        report['oci_contract_satisfied']=(result.returncode==0 and report['warning_observed'] and report['remaining_hook_invoked'] and not report['state_directory_retained'])
    except Exception as error:
        errors.append(public_error(error))
    finally:
        if cid and args.kind=='rune' and (root/'state'/cid).exists():
            try:
                code='import json,pathlib,sys;p=pathlib.Path(sys.argv[1]);v=json.loads(p.read_text());v["hooks"]=None;p.write_text(json.dumps(v))'
                call(['unshare','python3','-c',code,str(root/'state'/cid/'youki_config.json')])
            except Exception as error:errors.append(public_error(error))
        report['cleanup']=finish_resources(root,scoped,call,errors,Path(__file__).resolve().parents[1]/'work/smoke-recovery')
        report['runtime_scope']=summary(trace)
    report['error_count']=len(errors)
    report['scope']='Observed poststop failure/order/continuation; not conformance or complete parity'
    args.results.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'kind':args.kind,'delete_exit_code':report.get('delete_exit_code'),
                      'remaining_hook_invoked':report.get('remaining_hook_invoked'),'error_count':len(errors)}))
    if errors:raise ExceptionGroup('poststop investigation or private cleanup failed',errors)


if __name__=='__main__':main()
