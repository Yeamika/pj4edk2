#!/usr/bin/env python3
"""Measure virtual reset-to-PocketJS first submission on an explicit EFI.

Starts QEMU paused at reset, enables any CPU budget before resuming, and
records host wall time to serial markers. This does not measure physical
power-on, a board display's illumination, or cycle-accurate CPU performance.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import threading
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', type=Path, required=True)
    parser.add_argument('--efi', type=Path, required=True)
    parser.add_argument('--tools', type=Path, required=True)
    parser.add_argument('--cpu', default='cortex-a55')
    parser.add_argument('--size', default='1200x1920')
    parser.add_argument('--cpu-budget', type=float, default=100)
    parser.add_argument('--host-cpu', type=int)
    parser.add_argument('--timeout', type=float, default=90)
    args = parser.parse_args()
    if not 0 < args.cpu_budget <= 100 or args.timeout <= 0:
        parser.error('CPU budget must be in (0,100]; timeout must be positive')
    available = sorted(os.sched_getaffinity(0))
    host_cpu = available[0] if args.host_cpu is None else args.host_cpu
    if host_cpu not in available:
        parser.error('requested host CPU is unavailable')
    control_cpu = next((cpu for cpu in available if cpu != host_cpu), None)
    if control_cpu is None:
        parser.error('requires a separate host CPU for timing and budget control')
    # Keep serial polling, PNG conversion and the limiter off the guest CPU.
    os.sched_setaffinity(0, {control_cpu})
    run = args.run.resolve()
    if run.exists():
        parser.error('use a fresh run directory')
    run.mkdir(parents=True)
    scripts = Path(__file__).resolve().parent
    spec = importlib.util.spec_from_file_location('qemu_profile', scripts / 'qemu-profile.py')
    profile = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(profile)
    start = [sys.executable, str(scripts / 'qemu-smoke.py'), '--run', str(run),
             '--efi', str(args.efi.resolve()), '--tools', str(args.tools.resolve()),
             '--size', args.size, '--cpu', args.cpu, '--cpus', '1',
             '--memory', '1024', '--paused', '--ready-timeout', '20']
    report = {
        'cpu_model': args.cpu, 'vcpus': 1, 'memory_mib': 1024,
        'display': args.size, 'host_cpu': host_cpu, 'control_host_cpu': control_cpu,
        'cpu_budget_percent': args.cpu_budget,
        'cpu_budget_scope': 'enabled before virtual reset is resumed',
        'budget_method': '5 ms feedback with at most 5 ms of idle credit; unlimited at 100%',
        'efi_input': str(args.efi.resolve()),
        'efi_sha256': hashlib.sha256(args.efi.read_bytes()).hexdigest(),
        'start_command': start,
        'time_origin': 'host monotonic clock immediately before QMP cont from reset',
        'timing_scope': 'QEMU/AAVMF reset through first successful UI submission; not physical power-on',
        'poll_interval_ms': 5, 'markers': {},
    }
    done = threading.Event()
    limiter_ready = threading.Event()
    limiter_errors = []
    limiter = None
    pid = None
    error = None
    try:
        with (run / 'start.log').open('w') as output:
            subprocess.run(start, stdout=output, stderr=subprocess.STDOUT,
                           timeout=30, check=True,
                           preexec_fn=lambda: os.sched_setaffinity(0, {host_cpu}))
        pid = int((run / 'pid').read_text())
        status = profile.qmp(run, 'query-status')
        report['status_before_resume'] = status
        if status['running'] or status['status'] not in ['prelaunch', 'paused']:
            raise RuntimeError('guest was not paused at reset')
        initial_cpu = profile.cpu_seconds(pid)
        started = time.monotonic()

        def limit():
            stopped = False
            try:
                os.sched_setaffinity(0, {control_cpu})
                previous_time = time.monotonic()
                previous_cpu = profile.cpu_seconds(pid)
                credit = .005
                limiter_ready.set()
                while not done.is_set():
                    current_time = time.monotonic()
                    current_cpu = profile.cpu_seconds(pid)
                    # Firmware waits must not accumulate seconds of quota
                    # that let application startup run at full CPU speed.
                    credit = min(.005, credit
                                 + (current_time - previous_time) * args.cpu_budget / 100
                                 - (current_cpu - previous_cpu))
                    previous_time, previous_cpu = current_time, current_cpu
                    should_stop = credit < 0
                    if should_stop != stopped:
                        os.kill(pid, signal.SIGSTOP if should_stop else signal.SIGCONT)
                        stopped = should_stop
                    if done.wait(.005):
                        break
            except Exception as exc:
                limiter_errors.append(repr(exc))
                limiter_ready.set()
            finally:
                try:
                    os.kill(pid, signal.SIGCONT)
                except ProcessLookupError:
                    pass

        if args.cpu_budget < 100:
            limiter = threading.Thread(target=limit, daemon=True)
            limiter.start()
            if not limiter_ready.wait(2) or limiter_errors:
                raise RuntimeError(f'CPU limiter did not start: {limiter_errors}')
        # Exclude controller setup from the virtual boot interval.
        started = time.monotonic()
        initial_cpu = profile.cpu_seconds(pid)
        profile.qmp(run, 'cont')
        report['resume_command_return_s'] = time.monotonic() - started
        markers = {
            'app_entry': b'PJ4EDK2_ENTRY',
            'protocols_ready': b'PJ4EDK2_UEFI_READY',
            'first_submission': b'PJ4EDK2_FRAME_OK elapsed_ms=0 frames=1 ',
        }
        previous_poll_start = started
        maximum_poll_gap = 0
        while True:
            poll_started = time.monotonic()
            text = (run / 'serial.log').read_bytes()
            read_at = time.monotonic()
            maximum_poll_gap = max(maximum_poll_gap, read_at - previous_poll_start)
            for name, marker in markers.items():
                if name not in report['markers'] and marker in text:
                    report['markers'][name] = {
                        'detected_s': round(read_at - started, 6),
                        'lower_bound_s': round(previous_poll_start - started, 6),
                        'qemu_cpu_s': round(profile.cpu_seconds(pid) - initial_cpu, 4),
                    }
            if 'first_submission' in report['markers']:
                report['reset_to_first_submission_s'] = round(read_at - started, 6)
                report['qemu_cpu_s_to_first_submission'] = round(
                    profile.cpu_seconds(pid) - initial_cpu, 4)
                report['qemu_cpu_percent_to_first_submission'] = round(
                    100 * report['qemu_cpu_s_to_first_submission'] / (read_at - started), 2)
                break
            if read_at - started > args.timeout:
                raise TimeoutError('PocketJS first frame not observed')
            if limiter_errors:
                raise RuntimeError(limiter_errors)
            previous_poll_start = poll_started
            time.sleep(.005)
        report['maximum_poll_gap_ms'] = round(maximum_poll_gap * 1000, 3)
        serial = text.decode(errors='replace')
        report['app_run_to_first_submission_ms'] = int(
            re.search(r'run_to_first_submit_ms=(\d+)', serial)[1])
        entry = report['markers']['app_entry']
        first = report['markers']['first_submission']
        report['app_observed_cpu_percent'] = round(
            100 * (first['qemu_cpu_s'] - entry['qemu_cpu_s'])
            / (first['detected_s'] - entry['detected_s']), 2)
        report['first_tick_ms'] = int(re.search(r'tick_ms=(\d+)', serial)[1])
        report['ready'] = f'PJ4EDK2_UEFI_READY GOP={args.size}' in serial
        ppm = run / 'first-frame.ppm'
        png = run / 'first-frame.png'
        profile.qmp(run, 'screendump', {'filename': str(ppm)})
        report['screenshot_complete_s'] = round(time.monotonic() - started, 6)
        report['screenshot'] = {'path': png.name, **profile.png_from_ppm(ppm, png)}
        if not report['ready']:
            raise RuntimeError('requested GOP resolution not observed')
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
        if limiter_errors:
            report['limiter_errors'] = limiter_errors
            error = error or 'CPU limiter failed'
        (run / 'boot-profile.json').write_text(json.dumps(report, indent=2) + '\n')
    if error:
        raise SystemExit(error)
    print(json.dumps({key: report.get(key) for key in [
        'cpu_model', 'cpu_budget_percent', 'reset_to_first_submission_s',
        'app_run_to_first_submission_ms', 'qemu_cpu_percent_to_first_submission',
        'screenshot_complete_s', 'maximum_poll_gap_ms']}), flush=True)


if __name__ == '__main__':
    main()
