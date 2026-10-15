"""Private state-root binding for native OCI test invocations, including callbacks."""
import fcntl
import json
import os
from pathlib import Path
import stat
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
COMMANDS = {'checkpoint', 'create', 'delete', 'events', 'exec', 'features', 'kill',
            'list', 'pause', 'ps', 'restore', 'resume', 'run', 'spec', 'start', 'state', 'update'}
TRACE_BYTES = 64 * 1024


def root_arguments(argv, expected):
    forwarded = []
    values = []
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--":
            forwarded.extend(argv[index:])
            break
        if argument == '--root':
            index += 1
            if index == len(argv):
                raise ValueError('missing root value')
            values.append(argv[index])
        elif argument.startswith('--root='):
            values.append(argument.split('=', 1)[1])
        else:
            forwarded.append(argument)
        index += 1
    if len(values) > 1 or (values and values[0] != str(expected)):
        raise ValueError('unexpected runtime root')
    return ['--root=' + str(expected)] + forwarded, bool(values)


def append_event(path, event):
    fd = os.open(path, os.O_WRONLY | os.O_APPEND | os.O_CREAT | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK, 0o600)
    try:
        metadata = os.fstat(fd)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_uid != os.geteuid():
            raise ValueError("invalid runtime scope trace descriptor")
        fcntl.flock(fd, fcntl.LOCK_EX)
        encoded = (json.dumps(event, sort_keys=True) + '\n').encode()
        if os.fstat(fd).st_size + len(encoded) > TRACE_BYTES:
            raise ValueError('runtime scope trace exceeded bound')
        while encoded:
            written = os.write(fd, encoded)
            if not written:
                raise OSError("incomplete runtime scope trace write")
            encoded = encoded[written:]
    finally:
        os.close(fd)


def invoke(native, expected, trace, argv):
    operation = next((arg for arg in argv if arg in COMMANDS), 'metadata')
    try:
        forwarded, explicit = root_arguments(argv, expected)
        try:
            metadata = expected.parent.lstat()
            active = stat.S_ISDIR(metadata.st_mode) and metadata.st_uid == os.geteuid() and not metadata.st_mode & 0o077
        except FileNotFoundError:
            active = False
        allowed = active or operation in {'metadata', 'features'}
        append_event(trace, {'operation': operation, 'root_argument_present': explicit,
                             'root_bound': allowed, 'scope_active': active})
        if not allowed:
            raise ValueError('private runtime scope expired or changed')
        # Preserve the invocation PID, descriptors and stdio. This is fixture
        # configuration; rune still executes its native backend directly.
        os.execv(str(native), [str(native)] + forwarded)
    except (OSError, ValueError):
        print('private runtime scope rejected invocation; details withheld', file=sys.stderr)
        raise SystemExit(125) from None


def make_runtime(native, state_root):
    directory = ROOT / 'work/runtime-scopes'
    directory.mkdir(parents=True, exist_ok=True, mode=0o700)
    metadata = directory.lstat()
    if not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.geteuid() or metadata.st_mode & 0o077:
        raise RuntimeError('runtime scope fixture directory is insecure')
    scope = Path(tempfile.mkdtemp(prefix='scope-', dir=directory))
    launcher = scope / native.name
    trace = scope / 'requests.jsonl'
    launcher.write_text('#!/usr/bin/python3\nimport sys,pathlib\nsys.dont_write_bytecode=True\n'
        f'sys.path.insert(0, {str(ROOT / "scripts")!r})\nfrom smoke_scope import invoke\n'
        f'invoke(pathlib.Path({str(native)!r}), pathlib.Path({str(state_root)!r}), '
        f'pathlib.Path({str(trace)!r}), sys.argv[1:])\n')
    launcher.chmod(0o700)
    return launcher, trace


def summary(trace):
    if not trace.exists():
        return {'operations': {}, 'missing_root_arguments': {}, 'expired_scopes': {}}
    fd = os.open(trace, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
    with os.fdopen(fd, 'rb') as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise RuntimeError('runtime scope trace must be regular')
        fcntl.flock(stream, fcntl.LOCK_SH)
        payload = stream.read(TRACE_BYTES + 1)
    if len(payload) > TRACE_BYTES:
        raise RuntimeError('runtime scope trace exceeded bound')
    result = {'operations': {}, 'missing_root_arguments': {}, 'expired_scopes': {}}
    for line in payload.splitlines():
        event = json.loads(line)
        operation = event['operation']
        if operation not in COMMANDS | {'metadata'} or set(event) != {'operation', 'root_argument_present', 'root_bound', 'scope_active'}:
            raise RuntimeError('invalid runtime scope trace')
        if not all(isinstance(event[key], bool) for key in ['root_argument_present', 'root_bound', 'scope_active']):
            raise RuntimeError('invalid runtime scope trace fields')
        if not event['root_bound']:
            result['expired_scopes'][operation] = result['expired_scopes'].get(operation, 0) + 1
        result['operations'][operation] = result['operations'].get(operation, 0) + 1
        if not event['root_argument_present']:
            result['missing_root_arguments'][operation] = result['missing_root_arguments'].get(operation, 0) + 1
    return result
