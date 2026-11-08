#!/usr/bin/env python3
"""Measure continuous UI frame pacing using a QEMU-side GPU trace only.

The input EFI is copied unchanged. QEMU's log trace has no timestamps, so a
reader on a separate host CPU timestamps each display-flush line received
over a FIFO. These are virtual display submissions, not panel scanouts.
"""
import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import select
import signal
import subprocess
import sys
import threading
import time


class DisplayTrace:
    def __init__(self, run):
        self.path = run / 'display-trace.log'
        os.mkfifo(self.path)
        self.fd = os.open(self.path, os.O_RDONLY | os.O_NONBLOCK)
        self.finished = threading.Event()
        self.condition = threading.Condition()
        self.events = []
        self.error = None
        self.raw = (run / 'display-trace.txt').open('wb')
        self.output = (run / 'display-events.jsonl').open('w')
        self.thread = threading.Thread(target=self.read, daemon=True)
        self.thread.start()

    def read(self):
        pending = b''
        try:
            while not self.finished.is_set():
                if not select.select([self.fd], [], [], .05)[0]:
                    continue
                chunk = os.read(self.fd, 65536)
                received = time.monotonic_ns()
                if not chunk:
                    self.finished.wait(.005)
                    continue
                self.raw.write(chunk)
                self.raw.flush()
                lines = (pending + chunk).split(b'\n')
                pending = lines.pop()
                matched = [line for line in lines
                           if b'virtio_gpu_cmd_res_flush ' in line]
                for line in matched:
                    match = re.search(rb'res (\S+), w (\d+), h (\d+), x (\d+), y (\d+)', line)
                    if not match:
                        raise ValueError(f'unexpected GPU trace line: {line!r}')
                    event = {'host_ns': received, 'resource': match[1].decode(),
                             'width': int(match[2]), 'height': int(match[3]),
                             'x': int(match[4]), 'y': int(match[5]),
                             'events_in_read': len(matched)}
                    with self.condition:
                        self.events.append(event)
                        self.condition.notify_all()
                    self.output.write(json.dumps(event) + '\n')
                    self.output.flush()
        except Exception as exc:
            self.error = repr(exc)
        finally:
            self.raw.close()
            self.output.close()

    def count(self):
        with self.condition:
            return len(self.events)

    def after(self, index, timeout):
        with self.condition:
            if len(self.events) <= index:
                self.condition.wait(timeout)
            return list(self.events[index:])

    def close(self):
        self.finished.set()
        self.thread.join(timeout=2)
        os.close(self.fd)


def frame_stats(events):
    intervals = [(b['host_ns'] - a['host_ns']) / 1_000_000
                 for a, b in zip(events, events[1:])]
    if len(intervals) < 20 or any(value <= 0 for value in intervals):
        raise ValueError('insufficient or batched display timestamps')
    if any(event['events_in_read'] != 1 for event in events):
        raise ValueError('multiple display flushes received together; timing is ambiguous')
    ordered = sorted(intervals)
    count = len(ordered)
    p95 = ordered[math.ceil(.95 * count) - 1]
    slow = ordered[-math.ceil(.05 * count):]
    return {
        'frame_intervals': count,
        'duration_s': sum(intervals) / 1000,
        'average_fps': 1000 * count / sum(intervals),
        'p50_frame_ms': ordered[math.ceil(.50 * count) - 1],
        'p95_frame_ms': p95,
        'p95_inverse_fps': 1000 / p95,
        'five_percent_low_fps': 1000 * len(slow) / sum(slow),
        'five_percent_low_samples': len(slow),
        'maximum_frame_ms': max(intervals),
        'over_33_ms': sum(value > 1000 / 30 for value in intervals),
        'over_100_ms': sum(value > 100 for value in intervals),
        'over_500_ms': sum(value > 500 for value in intervals),
        'intervals_ms': intervals,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--efi', type=Path, required=True)
    parser.add_argument('--tools', type=Path, required=True)
    parser.add_argument('--cpu', default='cortex-a55')
    parser.add_argument('--cpu-budget', type=float, default=100)
    parser.add_argument('--size', default='1200x1920')
    parser.add_argument('--frames', type=int, default=101,
                        help='display submissions per continuous workload, including its first frame')
    parser.add_argument('--phase-timeout', type=float, default=180)
    args = parser.parse_args()
    if not 0 < args.cpu_budget <= 100 or args.frames < 21:
        parser.error('CPU budget must be in (0,100]; at least 21 frames required')
    available = sorted(os.sched_getaffinity(0))
    if len(available) < 2:
        parser.error('requires a separate host CPU for tracing and control')
    host_cpu, control_cpu = available[:2]
    os.sched_setaffinity(0, {control_cpu})
    run = args.run.resolve()
    if run.exists():
        parser.error('use a fresh run directory')
    run.mkdir(parents=True)
    scripts = Path(__file__).resolve().parent
    spec = importlib.util.spec_from_file_location('qemu_profile', scripts / 'qemu-profile.py')
    profile = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(profile)
    trace = DisplayTrace(run)
    done = threading.Event()
    limiter = None
    pid = None
    report = {
        'cpu_model': args.cpu, 'cpu_budget_percent': args.cpu_budget,
        'display': args.size, 'vcpus': 1, 'memory_mib': 1024,
        'host_cpu': host_cpu, 'control_host_cpu': control_cpu,
        'probe_scope': 'QEMU-side virtio GPU resource flush trace; no EFI instrumentation',
        'timestamp_scope': 'host monotonic receipt time of each trace line over FIFO',
        'low_definition': '1000 / mean of slowest ceil(5% * N) display frame intervals in ms',
        'p95_definition': 'nearest-rank 95th percentile of display frame intervals',
        'efi_input': str(args.efi.resolve()),
        'efi_sha256': hashlib.sha256(args.efi.read_bytes()).hexdigest(),
        'workloads': [],
    }
    error = None
    try:
        start = [sys.executable, str(scripts / 'qemu-smoke.py'), '--run', str(run),
                 '--efi', str(args.efi.resolve()), '--tools', str(args.tools.resolve()),
                 '--cpu', args.cpu, '--cpus', '1', '--memory', '1024',
                 '--size', args.size, '--trace-display', '--ready-timeout', '60']
        report['start_command'] = start
        with (run / 'start.log').open('w') as output:
            os.sched_setaffinity(0, {host_cpu})
            try:
                subprocess.run(start, stdout=output, stderr=subprocess.STDOUT,
                               timeout=75, check=True)
            finally:
                os.sched_setaffinity(0, {control_cpu})
        pid = int((run / 'pid').read_text())
        if args.cpu_budget < 100:
            limiter = threading.Thread(target=profile.throttle,
                                       args=(pid, args.cpu_budget, control_cpu, done), daemon=True)
            limiter.start()
        deadline = time.monotonic() + 30
        while 'PJB_FRAME_OK' not in (run / 'serial.log').read_text(errors='replace'):
            if trace.error or time.monotonic() > deadline:
                raise RuntimeError(trace.error or 'no first frame')
            time.sleep(.01)
        time.sleep(3)

        def workload(name, key):
            begin = trace.count()
            processed = begin
            inputs = []
            started = time.monotonic()
            initial_cpu = profile.cpu_seconds(pid)

            def send():
                at = time.monotonic_ns()
                profile.qmp(run, 'send-key', {'keys': [{'type': 'qcode', 'data': key}],
                                              'hold-time': 10})
                inputs.append(at)

            send()
            while processed - begin < args.frames:
                if trace.error:
                    raise RuntimeError(trace.error)
                if time.monotonic() - started > args.phase_timeout:
                    raise TimeoutError(f'{name}: only {processed - begin} display submissions')
                additions = trace.after(processed, .1)
                for event in additions:
                    processed += 1
                    if processed - begin >= args.frames:
                        break
                    send()
            elapsed = time.monotonic() - started
            events = trace.events[begin:processed]
            stats = frame_stats(events)
            record = {
                'name': name, 'input_key': key,
                'input_schedule': 'one key after each observed display submission, no intended idle gaps',
                'first_event_index': begin, 'last_event_index': processed - 1,
                'input_host_ns': inputs,
                'measured_host_s': elapsed,
                'input_to_first_submission_ms': (events[0]['host_ns'] - inputs[0]) / 1_000_000,
                'actual_cpu_percent': 100 * (profile.cpu_seconds(pid) - initial_cpu) / elapsed,
                'full_screen_submissions': sum(
                    event['width'] * event['height'] == math.prod(map(int, args.size.split('x')))
                    for event in events),
                **stats,
            }
            report['workloads'].append(record)
            ppm, png = run / f'{name}.ppm', run / f'{name}.png'
            profile.qmp(run, 'screendump', {'filename': str(ppm)})
            record['screenshot'] = {'path': png.name, **profile.png_from_ppm(ppm, png)}
            print(json.dumps({'cpu': args.cpu, 'budget': args.cpu_budget, 'workload': name,
                              **{key: record[key] for key in ['frame_intervals', 'average_fps',
                                  'p95_frame_ms', 'five_percent_low_fps', 'maximum_frame_ms',
                                  'actual_cpu_percent']}}), flush=True)

        workload('continuous-selection', 'down')
        time.sleep(3)
        workload('continuous-settings-toggle', 'f2')
        copied = run / 'esp/EFI/BOOT/BOOTAA64.EFI'
        report['guest_efi_sha256'] = hashlib.sha256(copied.read_bytes()).hexdigest()
        if report['guest_efi_sha256'] != report['efi_sha256']:
            raise RuntimeError('input EFI was changed')
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
                profile.qmp(run, 'quit')
            except Exception:
                try:
                    os.kill(pid, signal.SIGCONT)
                    os.kill(pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
        trace.close()
        report['trace_error'] = trace.error
        (run / 'frame-profile.json').write_text(json.dumps(report, indent=2) + '\n')
    if error:
        raise SystemExit(error)


if __name__ == '__main__':
    main()
