"""Control/filesystem fault tests; these do not prove kernel isolation."""
import json
import os
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest

from smoke_cleanup import finish_resources, state_summary, run_bounded, LIMIT, STATE_BYTES
from unittest.mock import patch


class CleanupTests(unittest.TestCase):
    def test_subprocess_timeout_withholds_arguments_and_output(self):
        failure = subprocess.TimeoutExpired(['private-path', 'private-id'], 1, output='private-workload')
        with patch('smoke_cleanup.subprocess.run', side_effect=failure):
            with self.assertRaises(RuntimeError) as caught:
                run_bounded(['private-path'])
        for private in ['private-path', 'private-id', 'private-workload']:
            self.assertNotIn(private, str(caught.exception))

    def test_optimized_python_cannot_skip_general_kernel_assertions(self):
        result = subprocess.run([sys.executable, '-O', str(Path(__file__).with_name('podman-smoke.py'))],
                                capture_output=True, text=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('assertions enabled', result.stderr)

    def test_enumeration_requests_full_ids_before_owned_removal(self):
        with tempfile.TemporaryDirectory(prefix='rune-smoke-ids-') as directory:
            root = Path(directory) / 'owned'
            root.mkdir()
            active = True
            errors = []
            def call(args):
                nonlocal active
                if args[0] == 'ps':
                    self.assertIn('--no-trunc', args)
                    return 'a' * 64 if active else ''
                if args[0] == 'rm':
                    self.assertEqual(args[-1], 'a' * 64)
                    active = False
                else:
                    root.rmdir()
                return ''
            finish_resources(root, Path('/synthetic-runtime'), call, errors, Path(directory) / 'recovery')
            self.assertFalse(errors)
            self.assertFalse(active)
            self.assertFalse(root.exists())

    def test_operation_and_cleanup_errors_survive_and_records_are_retained(self):
        with tempfile.TemporaryDirectory(prefix='rune-smoke-fault-') as directory:
            temp = Path(directory)
            root = temp / 'owned'
            records = root / 'state' / 'private-identifier'
            records.mkdir(parents=True)
            (records / 'state.json').write_text(json.dumps({'status': 'stopped', 'secret': 'private-workload'}))
            primary = RuntimeError('operation failure')
            errors = [primary]
            calls = []
            def call(args):
                calls.append(args[0])
                if args[0] == 'ps':
                    return 'a' * 64
                if args[0] == 'rm':
                    raise RuntimeError('cleanup failure')
                raise AssertionError('state must not be discarded')
            report = finish_resources(root, Path('/synthetic-runtime'), call, errors, temp / 'recovery')
            self.assertIs(errors[0], primary)
            self.assertTrue(any(str(error) == 'cleanup failure' for error in errors))
            self.assertTrue((records / 'state.json').is_file())
            self.assertTrue(report['resources_retained'])
            self.assertNotIn('unshare', calls)
            encoded = json.dumps(report)
            for private in ['private-identifier', 'private-workload', str(root), 'a' * 64]:
                self.assertNotIn(private, encoded)
            locator = next((temp / 'recovery').glob('*.json'))
            self.assertEqual(locator.stat().st_mode & 0o777, 0o600)
            self.assertEqual(json.loads(locator.read_text())['root'], str(root))

    def test_confirmed_cleanup_preserves_operation_error_and_removes_owned_tree(self):
        with tempfile.TemporaryDirectory(prefix='rune-smoke-clean-') as directory:
            temp = Path(directory)
            root = temp / 'owned'
            root.mkdir()
            errors = [RuntimeError('operation failure')]
            def call(args):
                if args[0] == 'ps':
                    return ''
                self.assertEqual(args[0], 'unshare')
                root.rmdir()
                return ''
            report = finish_resources(root, Path('/synthetic-runtime'), call, errors, temp / 'recovery')
            self.assertEqual(len(errors), 1)
            self.assertFalse(root.exists())
            self.assertFalse(report['resources_retained'])
            self.assertFalse((temp / 'recovery').exists())

    def test_residual_host_metadata_is_removed_and_failures_are_preserved(self):
        for removal_fails in [False, True]:
            with tempfile.TemporaryDirectory(prefix='rune-smoke-residual-') as directory:
                temp = Path(directory)
                root = temp / 'owned'
                root.mkdir()
                errors = []
                def call(args):
                    return ''
                if removal_fails:
                    with patch('smoke_cleanup.shutil.rmtree', side_effect=PermissionError('synthetic failure')):
                        report = finish_resources(root, Path('/synthetic-runtime'), call, errors, temp / 'recovery')
                    self.assertTrue(errors)
                    self.assertTrue(report['resources_retained'])
                else:
                    report = finish_resources(root, Path('/synthetic-runtime'), call, errors, temp / 'recovery')
                    self.assertFalse(errors)
                    self.assertFalse(root.exists())
                    self.assertFalse(report['resources_retained'])

    def test_diagnostics_are_bounded_and_do_not_follow_symlinks_or_fifos(self):
        with tempfile.TemporaryDirectory(prefix='rune-smoke-summary-') as directory:
            root = Path(directory)
            state = root / 'state'
            state.mkdir()
            target = root / 'private.json'
            target.write_text(json.dumps({'status': 'running', 'secret': 'workload'}))
            for index in range(LIMIT + 1):
                entry = state / f'private-{index}'
                entry.mkdir()
                if index % 3 == 0:
                    (entry / 'state.json').symlink_to(target)
                elif index % 3 == 1:
                    os.mkfifo(entry / 'state.json')
                else:
                    (entry / 'state.json').write_bytes(b' ' * (STATE_BYTES + 1))
            report = state_summary(root)
            self.assertEqual(report['entries'], LIMIT)
            self.assertTrue(report['truncated'])
            self.assertEqual(report['statuses'], {'unreadable': LIMIT})
            self.assertNotIn('private', json.dumps(report))
            self.assertNotIn('workload', json.dumps(report))

    def test_symlinked_state_root_and_unexpected_container_ids_fail_closed(self):
        with tempfile.TemporaryDirectory(prefix='rune-smoke-root-') as directory:
            temp = Path(directory)
            root = temp / 'owned'
            root.mkdir()
            (root / 'state').symlink_to(temp, target_is_directory=True)
            errors = []
            def call(args):
                self.assertEqual(args[0], 'ps')
                return '--unowned'
            report = finish_resources(root, Path('/synthetic-runtime'), call, errors, temp / 'recovery')
            self.assertTrue(errors)
            self.assertTrue(report['resources_retained'])
            self.assertTrue((root / 'state').is_symlink())


if __name__ == '__main__':
    unittest.main()
