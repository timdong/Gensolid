#!/usr/bin/env python3
"""Replay independent Creo STEP probes from WSL with structured evidence and owned cleanup."""
import argparse
import datetime
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import uuid

HERE = Path(__file__).resolve().parent
POWERSHELL = '/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe'


def winpath(path):
    return subprocess.check_output(['wslpath', '-w', str(Path(path).resolve())], text=True).strip()


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def ps(script):
    result = subprocess.run([POWERSHELL, '-NoProfile', '-Command', script],
                            capture_output=True, timeout=20, check=True)
    return result.stdout.decode('utf-8-sig').strip()


def processes(token):
    # Command lines are inspected locally only for our unique Java marker and never emitted.
    script = """$ErrorActionPreference='Stop'; @(Get-CimInstance Win32_Process | Where-Object { $_.CreationDate } | ForEach-Object {
      [PSCustomObject]@{pid=[int]$_.ProcessId; parent=[int]$_.ParentProcessId;
      name=$_.Name; created=$_.CreationDate.ToUniversalTime().ToString('o');
      marked=([bool]($_.CommandLine -like '*gensolid.probe.run_id=TOKEN*'))}
    }) | ConvertTo-Json -Compress""".replace('TOKEN', token)
    value = json.loads(ps(script) or '[]')
    return value if isinstance(value, list) else [value]


def discover_owned(snapshot, before, owned):
    for process in snapshot:
        identity = (process['pid'], process['created'])
        if identity not in before and process['marked'] and process['name'].lower() == 'java.exe':
            owned[process['pid']] = process
    # Include only descendants of the Java process or descendants observed earlier.
    current = {p['pid']: p for p in snapshot}
    changed = True
    while changed:
        changed = False
        for process in snapshot:
            if ((process['pid'], process['created']) not in before
                    and process['pid'] not in owned and process['parent'] in owned):
                parent = owned[process['parent']]
                # PID reuse cannot grant ownership of a process older than its parent.
                if (process['created'] >= parent['created'] and (parent['pid'] not in current
                        or current[parent['pid']]['created'] == parent['created'])):
                    owned[process['pid']] = process
                    changed = True


def stop_owned(owned, names=None):
    selected = [p for p in reversed(list(owned.values()))
                if names is None or p['name'].lower() in names]
    if not selected:
        return []
    def literal(value):
        return "'" + value.replace("'", "''") + "'"
    # Batch cleanup in one query, checking the original process identity before each stop.
    records = ','.join("[PSCustomObject]@{pid=" + str(p['pid']) + ";name=" + literal(p['name'])
                       + ";created=" + literal(p['created']) + "}" for p in selected)
    script = ("$ErrorActionPreference='Stop'; $stopped=@(); foreach ($o in @(" + records + ")) {"
              "$p=Get-CimInstance Win32_Process -Filter ('ProcessId='+$o.pid);"
              "if ($p -and $p.Name -eq $o.name -and $p.CreationDate.ToUniversalTime().ToString('o') -eq $o.created) {"
              "Stop-Process -Id $p.ProcessId -Force -ErrorAction Stop; $stopped+=$o.pid }};"
              "ConvertTo-Json -InputObject @($stopped) -Compress")
    stopped = json.loads(ps(script) or '[]')
    return [p for p in selected if p['pid'] in stopped]


def read_events(raw_path):
    events = []
    for line in raw_path.read_bytes().decode('utf-8', errors='replace').splitlines():
        # Native SDK output may precede the Java JSON on the same line.
        start = line.find('{"event":')
        if start < 0:
            continue
        try:
            event = json.loads(line[start:])
        except json.JSONDecodeError:
            continue
        if isinstance(event, dict) and isinstance(event.get('event'), str):
            events.append(event)
    return events


def run(args):
    config_path = Path(args.case).resolve()
    config = json.loads(config_path.read_text(encoding='utf-8'))
    if config.get('schema_version') != 1 or config.get('operation') != 'import_step':
        raise ValueError('Only schema_version=1, operation=import_step is supported')
    part_name = config['part_name']
    if not re.fullmatch(r'[a-z][a-z0-9_]{0,23}', part_name):
        raise ValueError('part_name must be a short lowercase Creo model name')
    source = (config_path.parent / config['input_step']).resolve()
    if not source.is_file() or source.suffix.lower() not in ('.step', '.stp'):
        raise ValueError('input_step must be an existing STEP file')
    if args.timeout <= 0:
        raise ValueError('timeout must be positive')
    creo = Path(args.creo_root).resolve()
    common = creo / 'Common Files'
    jars = [common / 'text/java/pfcasync.jar', common / 'text/java/otk.jar']
    java = common / 'x86e_win64/gpost/ncjlibs/jre/bin/java.exe'
    for path in [java, *jars, creo / 'Parametric/bin/parametric.exe']:
        if not path.is_file():
            raise FileNotFoundError(path)
    if args.dry_run:
        print(json.dumps({'case': config, 'input_sha256': digest(source),
                          'creo_root': str(creo), 'timeout_seconds': args.timeout}, ensure_ascii=False, indent=2))
        return 0
    run_id = uuid.uuid4().hex
    run_dir = Path(args.output_root).resolve() / (config['case_id'] + '-' + run_id[:12])
    run_dir.mkdir(parents=True, exist_ok=False)
    classes = run_dir / 'classes'
    classes.mkdir()
    probe_source = run_dir / 'CreoProbe.java'
    shutil.copyfile(HERE / 'CreoProbe.java', probe_source)
    runner_source_hash = digest(Path(__file__))
    compile_result = subprocess.run(['javac', '-encoding', 'UTF-8', '-cp', ':'.join(map(str, jars)),
                                     '-d', str(classes), str(probe_source)],
                                    capture_output=True, timeout=60)
    (run_dir / 'compile.log').write_bytes(compile_result.stdout + compile_result.stderr)
    if compile_result.returncode:
        raise RuntimeError(f'javac failed; see {run_dir / "compile.log"}')
    shutil.copyfile(source, run_dir / 'in.step')
    shutil.copyfile(config_path, run_dir / 'case.json')
    creo_command = f'"{winpath(creo / "Parametric/bin/parametric.exe")}" "{winpath(creo / "Parametric/bin/parametric.psf")}" -g:no_graphics -i:rpc_input'
    native_lib = winpath(common / 'x86e_win64/lib')
    command = [str(java), '-Dfile.encoding=UTF-8', f'-Dgensolid.probe.run_id={run_id}',
               f'-Djava.library.path={native_lib}', '-cp', ';'.join([winpath(classes), *map(winpath, jars)]),
               'CreoProbe', creo_command, winpath(run_dir), 'in.step', part_name]
    env = dict(os.environ)
    env['PATH'] = ';'.join([native_lib, winpath(common / 'x86e_win64/obj'),
                            r'C:\Windows\System32', r'C:\Windows'])
    env['WSLENV'] = 'PATH/w'
    raw_path = run_dir / 'stdout.bin'
    owned = {}
    before = {(p['pid'], p['created']) for p in processes(run_id)}
    started = time.monotonic()
    started_utc = datetime.datetime.now(datetime.timezone.utc).isoformat()
    timeout = False
    runner_error = None
    cleanup = []
    process = None
    try:
        with raw_path.open('wb') as raw:
            process = subprocess.Popen(command, stdout=raw, stderr=subprocess.STDOUT, env=env)
            while True:
                discover_owned(processes(run_id), before, owned)
                events = read_events(raw_path)
                ended = any(e['event'] == 'connection_ended' for e in events)
                if ended:
                    cleanup.extend(stop_owned(owned, {'nmsd.exe'}))
                if process.poll() is not None:
                    break
                if time.monotonic() - started >= args.timeout:
                    timeout = True
                    break
                time.sleep(0.5)
    except Exception as error:
        runner_error = f'{type(error).__name__}: {error}'
    finally:
        try:
            # Refresh ownership once before cleanup in case the last SDK call spawned a child.
            discover_owned(processes(run_id), before, owned)
            cleanup.extend(stop_owned(owned))
        except Exception as error:
            runner_error = (runner_error or '') + f'; cleanup: {type(error).__name__}: {error}'
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
    events = read_events(raw_path) if raw_path.exists() else []
    (run_dir / 'stdout.log').write_text(raw_path.read_bytes().decode('utf-8', errors='replace'), encoding='utf-8')
    (run_dir / 'events.jsonl').write_text(''.join(json.dumps(e, ensure_ascii=False) + '\n' for e in events), encoding='utf-8')
    exported = list(run_dir.glob('result.st*'))
    parts = list(run_dir.glob(part_name + '.prt.*'))
    completed = any(e['event'] == 'completion' and e.get('success') is True for e in events)
    success = (not timeout and runner_error is None and process is not None and process.returncode == 0
               and completed and len(exported) == 1 and bool(parts))
    repo = HERE.parent.parent
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
    patch = subprocess.check_output(['git', 'diff', '--ignore-space-at-eol'], cwd=repo)
    summary = {
        'schema_version': 1, 'run_id': run_id, 'case': config, 'started_utc': started_utc,
        'duration_seconds': time.monotonic() - started, 'success': success,
        'process_returncode': None if process is None else process.returncode,
        'timeout': timeout, 'runner_error': runner_error,
        'failure_events': [e for e in events if e['event'] == 'failure'],
        'features': [e for e in events if e['event'] == 'feature'],
        'mass_property': next((e for e in events if e['event'] == 'mass_property'), None),
        'creo_install_root': winpath(creo), 'sdk_jar_sha256': {p.name: digest(p) for p in jars},
        'gensolid_revision': revision, 'tracked_patch_sha256': hashlib.sha256(patch).hexdigest(),
        'probe_source_sha256': digest(probe_source),
        'runner_source_sha256': runner_source_hash,
        'artifacts': {p.name: digest(p) for p in [run_dir / 'in.step', *exported, *parts]},
        'owned_processes': list(owned.values()), 'cleanup_stopped': cleanup,
        'measurements': None, 'measurement_reason': 'Probe records interchange only; independent geometry inspection is a separate step',
    }
    (run_dir / 'summary.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'success': success, 'summary': str(run_dir / 'summary.json'),
                      'features': summary['features'], 'timeout': timeout, 'runner_error': runner_error}, ensure_ascii=False))
    return 0 if success else 1


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('case')
    parser.add_argument('--creo-root', default='/mnt/d/Program Files/PTC/Creo 10.0.8.0')
    parser.add_argument('--output-root', default=str(HERE.parent.parent / 'target/creo-probes'))
    parser.add_argument('--timeout', type=float, default=180)
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args()
    if args.dry_run:
        raise SystemExit(run(args))
    output_root = Path(args.output_root).resolve()
    output_root.mkdir(parents=True, exist_ok=True)
    # A Creo Name Server can be shared; keep probe runs sequential within this output root.
    with (output_root / '.probe.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        raise SystemExit(run(args))
