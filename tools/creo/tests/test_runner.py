"""Process ownership and event parsing regressions; never starts Creo."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('run_probe', Path(__file__).parents[1] / 'run_probe.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


def process(pid, parent, name, created, marked=False):
    return dict(pid=pid, parent=parent, name=name, created=created, marked=marked)


class RunnerTests(unittest.TestCase):
    def test_only_java_marker_grants_root_ownership(self):
        owned = {}
        snapshot = [process(1, 0, 'powershell.exe', '1', True),
                    process(2, 0, 'java.exe', '2', True),
                    process(3, 2, 'cmd.exe', '3'), process(4, 3, 'nmsd.exe', '4'),
                    process(5, 1, 'conhost.exe', '5'), process(6, 0, 'xtop.exe', '6')]
        probe.discover_owned(snapshot, set(), owned)
        self.assertEqual(set(owned), {2, 3, 4})

    def test_preexisting_and_reused_pids_do_not_grant_ownership(self):
        original = process(2, 0, 'java.exe', '2', True)
        owned = {2: original}
        probe.discover_owned([process(2, 0, 'other.exe', '8'), process(3, 2, 'nmsd.exe', '9')],
                             set(), owned)
        self.assertEqual(set(owned), {2})
        owned = {}
        probe.discover_owned([original], {(2, '2')}, owned)
        self.assertEqual(owned, {})

    def test_native_log_does_not_hide_json_events(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'stdout.bin'
            path.write_bytes(b'\xffnative\nnoise {"event":"feature","status_value":0}\r\n'
                             b'{"event":broken}\n{"event":"completion","success":true}\n')
            self.assertEqual([e['event'] for e in probe.read_events(path)], ['feature', 'completion'])

    def test_cleanup_is_batched_and_uses_identity(self):
        owned = {2: process(2, 0, 'java.exe', '2'), 3: process(3, 2, 'nmsd.exe', '3')}
        with patch.object(probe, 'ps', return_value='[3]') as command:
            self.assertEqual(probe.stop_owned(owned, {'nmsd.exe'}), [owned[3]])
            self.assertEqual(command.call_count, 1)
            self.assertIn('CreationDate', command.call_args.args[0])
            self.assertIn('pid=3', command.call_args.args[0])
            self.assertNotIn('pid=2', command.call_args.args[0])


if __name__ == '__main__':
    unittest.main()
