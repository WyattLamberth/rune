"""Bounded private diagnostics and cleanup for owned smoke-test resources."""
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import uuid

LIMIT = 32
STATE_BYTES = 64 * 1024
STATUSES = {'creating', 'created', 'running', 'paused', 'stopped'}


def public_error(error):
    if isinstance(error, OSError):
        return RuntimeError(f'{type(error).__name__} during filesystem operation (errno {error.errno}); details withheld')
    return error


def run_bounded(argv, timeout=45, text=False):
    try:
        return subprocess.run(argv, capture_output=True, timeout=timeout, text=text)
    except (subprocess.TimeoutExpired, OSError, UnicodeError):
        raise RuntimeError('smoke subprocess failed or timed out; details withheld') from None


def state_summary(root):
    result = {'entries': 0, 'truncated': False, 'statuses': {}, 'records': {}}
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    root_fd = os.open(root, flags)
    try:
        try:
            state_fd = os.open('state', flags, dir_fd=root_fd)
        except FileNotFoundError:
            return result
        try:
            with os.scandir(state_fd) as directory:
                for entry in directory:
                    if entry.name == '.rune-locks':
                        continue
                    if result['entries'] == LIMIT:
                        result['truncated'] = True
                        break
                    result['entries'] += 1
                    status = 'unreadable'
                    try:
                        container_fd = os.open(entry.name, flags, dir_fd=state_fd)
                    except OSError:
                        result['statuses'][status] = result['statuses'].get(status, 0) + 1
                        continue
                    try:
                        for name, category in [('state.json', 'state'), ('youki_config.json', 'cleanup'),
                                               ('rune-config.json', 'spec')]:
                            try:
                                metadata = os.stat(name, dir_fd=container_fd, follow_symlinks=False)
                                key = category if stat.S_ISREG(metadata.st_mode) else 'nonregular'
                                result['records'][key] = result['records'].get(key, 0) + 1
                            except FileNotFoundError:
                                pass
                        try:
                            fd = os.open('state.json', os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, dir_fd=container_fd)
                            with os.fdopen(fd, 'rb') as stream:
                                if stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
                                    payload = stream.read(STATE_BYTES + 1)
                                    if len(payload) <= STATE_BYTES:
                                        value = json.loads(payload)
                                        if isinstance(value, dict):
                                            recorded = value.get('status')
                                            status = recorded if isinstance(recorded, str) and recorded in STATUSES else 'not-recorded'
                        except (OSError, ValueError, TypeError):
                            pass
                    finally:
                        os.close(container_fd)
                    result['statuses'][status] = result['statuses'].get(status, 0) + 1
        finally:
            os.close(state_fd)
    finally:
        os.close(root_fd)
    return result


def retain_locator(root, runtime, directory):
    # This private recovery locator is never part of distributable evidence.
    directory.mkdir(parents=True, exist_ok=True, mode=0o700)
    metadata = directory.lstat()
    if not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.geteuid() or metadata.st_mode & 0o077:
        raise RuntimeError('private recovery directory is insecure')
    path = directory / f'{uuid.uuid4().hex}.json'
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC, 0o600)
    with os.fdopen(fd, 'w') as stream:
        json.dump({'root': str(root), 'runtime': str(runtime), 'format': 1}, stream)
        stream.flush()
        os.fsync(stream.fileno())


def finish_resources(root, runtime, call, errors, recovery_directory):
    report = {'before': {}, 'after': {}, 'resources_retained': False}
    start = len(errors)
    try:
        report['before'] = state_summary(root)
    except OSError:
        errors.append(RuntimeError('cannot inspect owned runtime state; details withheld'))
    try:
        identifiers = call(['ps', '--all', '--format', '{{.ID}}']).splitlines()
        if len(identifiers) > LIMIT or any(not re.fullmatch('[0-9a-f]{64}', identifier) for identifier in identifiers):
            raise RuntimeError('unexpected owned-container enumeration')
        for identifier in identifiers:
            try:
                call(['rm', '--force', identifier])
            except Exception as error:
                errors.append(public_error(error))
        if call(['ps', '--all', '--format', '{{.ID}}']):
            errors.append(RuntimeError('owned containers remained after cleanup'))
    except Exception as error:
        errors.append(public_error(error))
    try:
        report['after'] = state_summary(root)
        if report['after']['entries']:
            errors.append(RuntimeError('runtime ownership records remained after cleanup'))
    except OSError:
        errors.append(RuntimeError('cannot confirm runtime state cleanup; details withheld'))
    if len(errors) == start:
        try:
            call(['unshare', 'python3', '-c', 'import shutil,sys; shutil.rmtree(sys.argv[1])', str(root)])
            # Podman can recreate host-owned metadata while finishing unshare.
            # Confirm removal; never silently ignore remaining cleanup errors.
            if root.exists():
                metadata = root.lstat()
                if not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != os.geteuid():
                    raise RuntimeError('disposable root changed ownership or type')
                shutil.rmtree(root)
            if root.exists():
                raise RuntimeError('disposable tree remained after cleanup')
        except Exception as error:
            errors.append(public_error(error))
    if len(errors) != start:
        report['resources_retained'] = root.exists()
        if report['resources_retained']:
            try:
                retain_locator(root, runtime, recovery_directory)
            except OSError:
                errors.append(RuntimeError('cannot save private recovery locator; details withheld'))
            except RuntimeError as error:
                errors.append(public_error(error))
    return report
