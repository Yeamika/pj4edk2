#!/usr/bin/env python3
"""Exercise a pinned EFI at one CPU model and a measured host CPU budget.

TCG models instruction features, not silicon throughput. A reduced CPU budget
is a separate scheduling stress test, never a Cortex frequency conversion.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import socket
import statistics
import struct
import subprocess
import sys
import threading
import time
import zlib


def qmp(run, command, arguments=None):
    with socket.socket(socket.AF_UNIX) as connection:
        connection.settimeout(10)
        connection.connect(str(run / 'qmp.sock'))
        stream = connection.makefile('rwb', buffering=0)
        stream.readline()
        for name, args in [('qmp_capabilities', None), (command, arguments)]:
            request = {'execute': name}
            if args is not None:
                request['arguments'] = args
            stream.write((json.dumps(request) + '\n').encode())
            while True:
                reply = json.loads(stream.readline())
                if 'error' in reply:
                    raise RuntimeError(reply)
                if 'return' in reply:
                    break
        return reply['return']


def png_from_ppm(source, destination):
    with source.open('rb') as stream:
        assert stream.readline().strip() == b'P6'
        dimensions = stream.readline()
        while dimensions.startswith(b'#'):
            dimensions = stream.readline()
        width, height = map(int, dimensions.split())
        assert stream.readline().strip() == b'255'
        pixels = stream.read()
    assert len(pixels) == width * height * 3

    def chunk(kind, data):
        return (struct.pack('>I', len(data)) + kind + data
                + struct.pack('>I', zlib.crc32(kind + data)))

    rows = b''.join(b'\0' + pixels[y * width * 3:(y + 1) * width * 3]
                    for y in range(height))
    destination.write_bytes(
        b'\x89PNG\r\n\x1a\n'
        + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
        + chunk(b'IDAT', zlib.compress(rows, 6)) + chunk(b'IEND', b''))
    return {'width': width, 'height': height,
            'pixel_sha256': hashlib.sha256(pixels).hexdigest()}


def cpu_seconds(pid):
    # Fields after the parenthesised comm begin with field 3 (state).
    fields = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
    return (int(fields[11]) + int(fields[12])) / os.sysconf('SC_CLK_TCK')


def throttle(pid, percentage, control_cpu, finished):
    # Feedback measures all QEMU threads. A sleeping duty-cycle controller
    # alone can overshoot when it competes with the vCPU on the same host CPU.
    os.sched_setaffinity(0, {control_cpu})
    started = time.monotonic()
    initial = cpu_seconds(pid)
    stopped = False
    try:
        while not finished.is_set():
            allowed = (time.monotonic() - started) * percentage / 100 + 0.005
            should_stop = cpu_seconds(pid) - initial > allowed
            if should_stop != stopped:
                os.kill(pid, signal.SIGSTOP if should_stop else signal.SIGCONT)
                stopped = should_stop
            if finished.wait(.005):
                break
    except ProcessLookupError:
        pass
    finally:
        try:
            os.kill(pid, signal.SIGCONT)
        except ProcessLookupError:
            pass


def samples(text):
    result = []
    for line in text.splitlines():
        if 'PJB_FRAME_SLOW' not in line:
            continue
        row = {key: int(value) for key, value in
               re.findall(r'(\w+)=([0-9]+)', line)}
        stages = re.search(r'stages_ms=\[([0-9, ]+)\]', line)
        if stages:
            row['stages_ms'] = list(map(int, stages[1].split(',')))
        result.append(row)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--efi', type=Path, required=True)
    parser.add_argument('--tools', type=Path, required=True)
    parser.add_argument('--cpu', default='cortex-a55')
    parser.add_argument('--size', default='1200x1920')
    parser.add_argument('--cpu-budget', type=float, default=100)
    parser.add_argument('--host-cpu', type=int)
    parser.add_argument('--settle', type=float, default=2)
    args = parser.parse_args()
    if not 0 < args.cpu_budget <= 100 or args.settle <= 0:
        parser.error('CPU budget must be in (0,100]; settle must be positive')
    available = os.sched_getaffinity(0)
    host_cpu = min(available) if args.host_cpu is None else args.host_cpu
    if host_cpu not in available:
        parser.error('requested host CPU is unavailable')
    control_cpu = next((cpu for cpu in sorted(available) if cpu != host_cpu), host_cpu)
    os.sched_setaffinity(0, {host_cpu})
    run = args.run.resolve()
    if run.exists():
        parser.error('use a fresh run directory')
    run.mkdir(parents=True)
    start_log = (run / 'start.log').open('w')
    helper = Path(__file__).with_name('qemu-smoke.py')
    start = [sys.executable, str(helper), '--run', str(run),
             '--efi', str(args.efi.resolve()), '--tools', str(args.tools.resolve()),
             '--size', args.size, '--cpu', args.cpu, '--cpus', '1',
             '--memory', '1024', '--ready-timeout', '100']
    report = {
        'cpu_model': args.cpu, 'vcpus': 1, 'memory_mib': 1024,
        'display': args.size, 'host_cpu': host_cpu,
        'cpu_budget_percent': args.cpu_budget,
        'budget_method': 'unlimited' if args.cpu_budget == 100 else
                         'SIGSTOP/SIGCONT CPU-time feedback, 5 ms control interval',
        'control_host_cpu': control_cpu if args.cpu_budget < 100 else None,
        'efi_input': str(args.efi.resolve()),
        'efi_sha256': hashlib.sha256(args.efi.read_bytes()).hexdigest(),
        'timing_scope': 'QEMU guest timer; not equivalent physical CPU timing',
        'sample_scope': 'existing firmware logs at most one slow tick per guest second',
        'start_command': start, 'steps': [],
    }
    done = threading.Event()
    limiter = None
    pid = None
    error = None
    try:
        subprocess.run(start, stdout=start_log, stderr=subprocess.STDOUT,
                       timeout=115, check=True)
        pid = int((run / 'pid').read_text())
        if args.cpu_budget < 100:
            limiter = threading.Thread(target=throttle,
                                       args=(pid, args.cpu_budget, control_cpu, done), daemon=True)
            limiter.start()
        deadline = time.monotonic() + 30
        while 'PJB_FRAME_OK' not in (run / 'serial.log').read_text(errors='replace'):
            if time.monotonic() > deadline:
                raise TimeoutError('no initial rendered frame')
            time.sleep(.1)
        time.sleep(args.settle)
        measured_at = time.monotonic()
        initial_cpu = cpu_seconds(pid)

        def step(name, key=None, capture=False, pause=None):
            before = (run / 'serial.log').stat().st_size
            started_at = time.monotonic()
            if key:
                qmp(run, 'send-key', {'keys': [{'type': 'qcode', 'data': key}],
                                     'hold-time': 100})
            time.sleep(args.settle if pause is None else pause)
            record = {'name': name, 'key': key,
                      'host_elapsed_s': round(time.monotonic() - started_at, 4)}
            if capture:
                ppm = run / (name + '.ppm')
                png = run / (name + '.png')
                qmp(run, 'screendump', {'filename': str(ppm)})
                record['screenshot'] = {'path': png.name,
                                        **png_from_ppm(ppm, png)}
            text = (run / 'serial.log').read_bytes()[before:].decode(errors='replace')
            record['slow_samples'] = samples(text)
            report['steps'].append(record)
            print(name, len(record['slow_samples']), 'slow samples', flush=True)

        step('idle', capture=True)
        step('select-down', 'down', True)
        step('select-up', 'up', True)
        step('select-down-again', 'down')
        step('select-up-again', 'up')
        step('settings-open', 'f2', True)
        step('settings-close', 'f2', True)
        step('boot-arm', 'ret', True)
        step('boot-animation', 'ret', pause=max(3, args.settle))
        report['host_measured_s'] = round(time.monotonic() - measured_at, 4)
        report['qemu_cpu_s'] = round(cpu_seconds(pid) - initial_cpu, 4)
        report['qemu_cpu_percent_of_one_host_cpu'] = round(
            100 * report['qemu_cpu_s'] / report['host_measured_s'], 2)
        text = (run / 'serial.log').read_text(errors='replace')
        report['ready'] = f'PJB_UEFI_READY GOP={args.size}' in text
        report['boot_request'] = 'PJB_BOOT_REQUEST' in text
        report['boot_scope'] = 'animation and request only; no OS loader installed'
        report['logged_slow_samples'] = samples(text)
        measured = [row for item in report['steps'] for row in item['slow_samples']]
        report['measured_slow_sample_count'] = len(measured)
        if measured:
            report['sampled_slow_tick_median_ms'] = statistics.median(
                row['tick_ms'] for row in measured)
            report['sampled_slow_tick_max_ms'] = max(row['tick_ms'] for row in measured)
        hashes = {item['name']: item['screenshot']['pixel_sha256']
                  for item in report['steps'] if 'screenshot' in item}
        report['selection_changed_pixels'] = hashes['idle'] != hashes['select-down']
        report['settings_changed_pixels'] = hashes['idle'] != hashes['settings-open']
        if not all(report[key] for key in ['ready', 'boot_request',
                                          'selection_changed_pixels', 'settings_changed_pixels']):
            raise RuntimeError('UI exercise did not reach all required states')
    except Exception as exc:
        error = repr(exc)
        report['error'] = error
    finally:
        done.set()
        if limiter:
            limiter.join(timeout=2)
        if pid is None and (run / 'pid').exists():
            pid = int((run / 'pid').read_text())
        if pid is not None:
            try:
                qmp(run, 'quit')
            except (OSError, RuntimeError, ValueError):
                try:
                    os.kill(pid, signal.SIGCONT)
                    os.kill(pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
        start_log.close()
        (run / 'profile.json').write_text(json.dumps(report, indent=2) + '\n')
    if error:
        raise SystemExit(error)
    print(json.dumps({key: report.get(key) for key in
          ['cpu_model', 'cpu_budget_percent', 'qemu_cpu_percent_of_one_host_cpu',
           'measured_slow_sample_count', 'sampled_slow_tick_median_ms',
           'sampled_slow_tick_max_ms', 'boot_request']}), flush=True)


if __name__ == '__main__':
    main()
