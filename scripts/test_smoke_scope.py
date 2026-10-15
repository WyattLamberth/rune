"""Fixture contract tests; native process/isolation evidence comes from kernel suites."""
from pathlib import Path
import tempfile
import contextlib
import io
import os
import subprocess
import json
import unittest
from unittest.mock import patch

from smoke_scope import root_arguments, invoke, summary, make_runtime


class ScopeTests(unittest.TestCase):
    def test_callback_root_is_bound_and_conflicting_or_duplicate_roots_fail(self):
        expected = Path('/synthetic/state')
        forwarded, explicit = root_arguments(['delete', '--force', 'synthetic'], expected)
        self.assertFalse(explicit)
        self.assertEqual(forwarded, ['--root=/synthetic/state', 'delete', '--force', 'synthetic'])
        for argv in [['--root', '/other', 'delete'], ['--root'],
                     ['--root=/synthetic/state', '--root=/synthetic/state', 'delete']]:
            with self.assertRaises(ValueError):
                root_arguments(argv, expected)
        self.assertTrue(root_arguments(['--root', str(expected), 'state', 'synthetic'], expected)[1])

    def test_exec_keeps_native_invocation_and_trace_contains_no_private_values(self):
        with tempfile.TemporaryDirectory(prefix='rune-scope-contract-') as directory:
            root = Path(directory)
            trace = root / 'trace'
            with patch('smoke_scope.os.execv', side_effect=RuntimeError('exec intercepted')) as execute:
                with self.assertRaisesRegex(RuntimeError, 'exec intercepted'):
                    invoke(Path('/synthetic-native'), root / 'state', trace, ['delete', '--force', 'private-id'])
            self.assertEqual(execute.call_args.args[1][1], '--root=' + str(root / 'state'))
            report = summary(trace)
            self.assertEqual(report['missing_root_arguments'], {'delete': 1})
            self.assertNotIn('private-id', trace.read_text())
            self.assertNotIn(str(root), trace.read_text())

    def test_expired_scope_and_fifo_trace_never_invoke_native_runtime(self):
        with tempfile.TemporaryDirectory(prefix='rune-scope-reject-') as directory:
            root = Path(directory)
            for expected, trace in [(root / 'absent' / 'state', root / 'trace'),
                                     (root / 'state', root / 'fifo')]:
                if trace.name == 'fifo':
                    os.mkfifo(trace)
                with patch('smoke_scope.os.execv') as execute, contextlib.redirect_stderr(io.StringIO()):
                    with self.assertRaises(SystemExit) as caught:
                        invoke(Path('/synthetic-native'), expected, trace, ['delete', 'private-id'])
                    self.assertEqual(caught.exception.code, 125)
                execute.assert_not_called()
            self.assertEqual(summary(root / 'trace')['expired_scopes'], {'delete': 1})

    def test_native_fixture_preserves_pid_and_inherited_descriptor(self):
        with tempfile.TemporaryDirectory(prefix='rune-scope-process-') as directory:
            root = Path(directory)
            native = root / 'native-fixture'
            native.write_text('#!/usr/bin/python3\nimport json,os,sys\n'
                'fd=int(sys.argv[-1])\n'
                'print(json.dumps({"pid":os.getpid(),"inherited":os.read(fd,32)==b"synthetic-marker"}))\n')
            native.chmod(0o700)
            launcher, trace = make_runtime(native, root / 'state')
            with tempfile.TemporaryFile() as descriptor:
                descriptor.write(b'synthetic-marker'); descriptor.seek(0)
                process = subprocess.Popen([str(launcher), 'state', str(descriptor.fileno())],
                    pass_fds=(descriptor.fileno(),), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                output, _ = process.communicate(timeout=5)
            value = json.loads(output)
            self.assertTrue(process.returncode == 0 and value['pid'] == process.pid and value['inherited'],
                            'native fixture changed process or descriptor identity')
            self.assertEqual(summary(trace)['operations'], {'state': 1})
            launcher.unlink(); trace.unlink(); launcher.parent.rmdir()


if __name__ == '__main__':
    unittest.main()
