#!/usr/bin/env python3
"""Sample guest stacks through a local QEMU debug socket, without EFI changes.

This deliberately pauses the guest. Its timings are not FPS results; run the
display-frame profiler separately. Requires private GDB, pygdbmi and the
standard FlameGraph renderer, provided through explicit arguments.
"""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import queue
import random
import re
import signal
import subprocess
import sys
import threading
import time


class GdbMI:
    def __init__(self, command, env, output, parser):
        self.process = subprocess.Popen(command, env=env, stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                        text=True, bufsize=1)
        self.lines = queue.Queue()
        self.output = output.open('w')
        self.parser = parser
        self.token = 0
        self.stopped = True
        self.records = []

        def read():
            for line in self.process.stdout:
                self.lines.put(line)
            self.lines.put(None)

        self.reader = threading.Thread(target=read, daemon=True)
        self.reader.start()

    def next(self, timeout=15):
        line = self.lines.get(timeout=timeout)
        if line is None:
            raise RuntimeError('GDB exited')
        self.output.write(line)
        self.output.flush()
        try:
            record = self.parser(line.strip())
        except Exception:
            record = {'type': 'unparsed', 'payload': line}
        self.records.append(record)
        if record.get('type') == 'notify':
            if record.get('message') == 'stopped':
                self.stopped = True
            elif record.get('message') == 'running':
                self.stopped = False
        return record

    def request(self, command):
        self.token += 1
        token = self.token
        self.output.write(f'> {token}{command}\n')
        self.process.stdin.write(f'{token}{command}\n')
        self.process.stdin.flush()
        while True:
            record = self.next()
            if record.get('type') == 'result' and record.get('token') == token:
                if record['message'] == 'error':
                    raise RuntimeError(record)
                return record.get('payload')

    def console(self, command):
        before = len(self.records)
        self.request('-interpreter-exec console ' + json.dumps(command))
        return ''.join(str(r.get('payload', '')) for r in self.records[before:]
                       if r.get('type') in ['console', 'log'])

    def resume(self):
        self.request('-exec-continue')
        self.stopped = False

    def pause(self):
        if not self.stopped:
            self.request('-exec-interrupt')
            while not self.stopped:
                self.next()

    def close(self):
        try:
            self.request('-gdb-exit')
        except Exception:
            self.process.terminate()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        self.output.close()


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['run', 'efi', 'elf', 'tools', 'gdb-root', 'flamegraph', 'python-libs']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--samples', type=int, default=400)
    parser.add_argument('--period-ms', type=float, default=25)
    args = parser.parse_args()
    if args.samples < 1 or args.period_ms <= 0:
        parser.error('positive sample count and period required')
    sys.path.insert(0, str(args.python_libs.resolve()))
    from pygdbmi.gdbmiparser import parse_response
    available = sorted(os.sched_getaffinity(0))
    if len(available) < 2:
        parser.error('requires separate guest and sampling CPUs')
    host_cpu, control_cpu = available[:2]
    os.sched_setaffinity(0, {control_cpu})
    run = args.run.resolve()
    if run.exists():
        parser.error('use a fresh run directory')
    run.mkdir(parents=True)
    scripts = Path(__file__).resolve().parent
    profile = module('interaction_profile', scripts / 'qemu-profile.py')
    frames = module('display_profile', scripts / 'qemu-frame-profile.py')
    trace = frames.DisplayTrace(run)
    report = {'scope': 'QEMU guest-stack sampling only; no EFI changes or device operations',
              'cpu_model': 'cortex-a55', 'cpu_budget_percent': 100,
              'display': '1200x1920', 'host_cpu': host_cpu, 'control_host_cpu': control_cpu,
              'efi_input': str(args.efi.resolve()),
              'efi_sha256': hashlib.sha256(args.efi.read_bytes()).hexdigest(),
              'elf_input': str(args.elf.resolve()),
              'elf_sha256': hashlib.sha256(args.elf.read_bytes()).hexdigest(),
              'sampling': 'randomised execution intervals, GDB interrupt and stack unwind',
              'period_ms': args.period_ms, 'samples_requested_per_workload': args.samples,
              'fps_scope': 'debugger-paused profiling run; do not use for FPS', 'workloads': []}
    pid = None
    gdb = None
    error = None
    driver_done = threading.Event()
    driver = None
    try:
        start = [sys.executable, str(scripts / 'qemu-smoke.py'), '--run', str(run),
                 '--efi', str(args.efi.resolve()), '--tools', str(args.tools.resolve()),
                 '--cpu', 'cortex-a55', '--size', '1200x1920', '--trace-display', '--gdb']
        with (run / 'start.log').open('w') as output:
            os.sched_setaffinity(0, {host_cpu})
            try:
                subprocess.run(start, stdout=output, stderr=subprocess.STDOUT,
                               check=True, timeout=60)
            finally:
                os.sched_setaffinity(0, {control_cpu})
        pid = int((run / 'pid').read_text())
        deadline = time.monotonic() + 30
        while 'PJ4EDK2_FRAME_OK' not in (run / 'serial.log').read_text(errors='replace'):
            if time.monotonic() > deadline:
                raise TimeoutError('no first UI frame')
            time.sleep(.01)
        time.sleep(2)
        env = os.environ.copy()
        env['LD_LIBRARY_PATH'] = ':'.join(map(str, [
            args.gdb_root.resolve() / 'usr/lib/x86_64-linux-gnu',
            args.tools.resolve() / 'usr/lib/x86_64-linux-gnu',
            Path(sys.base_prefix) / 'lib']))
        env['PYTHONHOME'] = sys.base_prefix
        env['DEBUGINFOD_URLS'] = ''
        command = [str(args.gdb_root.resolve() / 'usr/bin/gdb-multiarch'),
                   '--nx', '--quiet', '--interpreter=mi2',
                   '--data-directory=' + str(args.gdb_root.resolve() / 'usr/share/gdb')]
        gdb = GdbMI(command, env, run / 'gdb-mi.log', parse_response)
        gdb.request('-gdb-set pagination off')
        gdb.request('-gdb-set confirm off')
        gdb.request('-gdb-set mi-async on')
        gdb.console('set debuginfod enabled off')
        # Attach during a real redraw, rather than a nested firmware Stall()
        # call whose PC and LR may both belong to AAVMF.
        profile.qmp(run, 'send-key', {'keys': [{'type': 'qcode', 'data': 'down'}],
                                     'hold-time': 10})
        time.sleep(.07)
        gdb.request('-target-select remote ' + str(run / 'gdb.sock'))
        # Match the loaded image against its original entry instructions.
        # Header and symbol reads affect only the disposable guest debugger.
        locate = run / 'locate-image.py'
        locate.write_text('''import gdb, struct
data = open(EFI_PATH, 'rb').read()
pe = struct.unpack_from('<I', data, 0x3c)[0]
entry = struct.unpack_from('<I', data, pe + 24 + 16)[0]
size = struct.unpack_from('<I', data, pe + 24 + 56)[0]
count = struct.unpack_from('<H', data, pe + 6)[0]
section = pe + 24 + struct.unpack_from('<H', data, pe + 20)[0]
for i in range(count):
    virtual_size, address, raw_size, raw = struct.unpack_from('<IIII', data, section+i*40+8)
    if address <= entry < address + virtual_size:
        signature = data[raw + entry-address:raw + entry-address+32]
inferior = gdb.selected_inferior()
base = None
for candidate in [int(gdb.parse_and_eval('$pc')), int(gdb.parse_and_eval('$x30'))]:
    print('PJ4EDK2_IMAGE_CANDIDATE=' + hex(candidate))
    for page in range(candidate & ~4095, max(0x40000000, candidate-size-4096), -4096):
        try:
            if inferior.read_memory(page,2).tobytes() != b'MZ': continue
            if inferior.read_memory(page+entry,32).tobytes() == signature:
                base = page; break
        except gdb.MemoryError: pass
    if base is not None: break
if base is None: raise RuntimeError('cannot locate matching EFI image')
print('PJ4EDK2_IMAGE_BASE=' + hex(base))
'''.replace('EFI_PATH', repr(str(args.efi.resolve()))))
        output = gdb.console('source ' + str(locate))
        match = re.search(r'PJ4EDK2_IMAGE_BASE=(0x[0-9a-f]+)', output)
        if not match:
            raise RuntimeError('matching EFI image base was not found: ' + output)
        base = int(match[1], 16)
        report['loaded_image_base'] = hex(base)
        gdb.console('symbol-file -o ' + hex(base) + ' ' + str(args.elf.resolve()))
        generator = random.Random(20261104)
        samples = (run / 'stack-samples.jsonl').open('w')

        def workload(name, key):
            driver_done.clear()
            cursor = trace.count()
            driver_errors = []

            def send():
                profile.qmp(run, 'send-key', {'keys': [{'type': 'qcode', 'data': key}],
                                             'hold-time': 10})

            def drive():
                nonlocal cursor
                try:
                    send()
                    while not driver_done.is_set():
                        additions = trace.after(cursor, .05)
                        for event in additions:
                            cursor += 1
                            if not driver_done.is_set():
                                send()
                except Exception as exc:
                    driver_errors.append(repr(exc))

            nonlocal driver
            driver = threading.Thread(target=drive, daemon=True)
            driver.start()
            stacks = Counter()
            leaf = Counter()
            depths = Counter()
            boundaries = Counter()
            for index in range(args.samples):
                if driver_errors or trace.error:
                    raise RuntimeError(driver_errors or trace.error)
                gdb.resume()
                execution_ms = args.period_ms * generator.uniform(.6, 1.4)
                time.sleep(execution_ms / 1000)
                gdb.pause()
                raw_stack = gdb.request('-stack-list-frames 0 63')['stack']
                if not raw_stack:
                    raise RuntimeError('empty guest stack')
                # This assembly trampoline switches stacks. Frames inferred
                # outside it are not reliable callers of the application.
                stack = []
                boundary = 'unwind ended before known stack-switch boundary'
                for frame in raw_stack:
                    stack.append(frame)
                    if frame.get('func') == 'PjbOnStack':
                        boundary = 'PjbOnStack'
                        break
                sample = {'workload': name, 'index': index, 'execution_interval_ms': execution_ms,
                          'host_ns': time.monotonic_ns(), 'frames': stack,
                          'raw_frames': raw_stack, 'stack_boundary': boundary}
                samples.write(json.dumps(sample) + '\n')
                samples.flush()
                names = []
                for frame in stack:
                    symbol = frame.get('func', '[unresolved ' + frame['addr'] + ']')
                    symbol = re.sub(r'\[[0-9a-f]{12,}\]', '', symbol)
                    names.append(symbol.replace(';', ','))
                leaf[names[0]] += 1
                stacks[';'.join(reversed(names))] += 1
                depths[len(names)] += 1
                boundaries[boundary] += 1
                if (index + 1) % 100 == 0:
                    print(name, index + 1, 'stack samples', flush=True)
            driver_done.set()
            driver.join(timeout=3)
            folded = run / (name + '.folded')
            folded.write_text(''.join(stack + ' ' + str(count) + '\n'
                                      for stack, count in sorted(stacks.items())))
            svg = run / (name + '.svg')
            with svg.open('w') as output:
                subprocess.run(['perl', str(args.flamegraph.resolve()), '--width', '1800',
                                '--title', 'PocketJS QEMU: ' + name,
                                '--subtitle', 'Guest stack samples; debugger pauses excluded from FPS tests',
                                '--countname', 'samples', str(folded)], stdout=output, check=True)
            record = {'name': name, 'samples': sum(stacks.values()),
                      'stack_depths': dict(sorted(depths.items())),
                      'stack_boundaries': dict(boundaries),
                      'leaf_samples': dict(leaf.most_common()), 'folded': folded.name, 'svg': svg.name}
            report['workloads'].append(record)
            print(name, 'top leaves:', leaf.most_common(8), flush=True)

        workload('continuous-selection', 'down')
        gdb.resume()
        time.sleep(2)
        gdb.pause()
        workload('continuous-settings-toggle', 'f2')
        samples.close()
        report['guest_efi_sha256'] = hashlib.sha256(
            (run / 'esp/EFI/BOOT/BOOTAA64.EFI').read_bytes()).hexdigest()
        if report['guest_efi_sha256'] != report['efi_sha256']:
            raise RuntimeError('EFI changed')
    except Exception as exc:
        error = repr(exc)
        report['error'] = error
    finally:
        driver_done.set()
        if driver:
            driver.join(timeout=3)
        if gdb:
            gdb.close()
        if pid is None and (run / 'pid').exists():
            pid = int((run / 'pid').read_text())
        if pid is not None:
            try:
                profile.qmp(run, 'quit')
            except Exception:
                try:
                    os.kill(pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
        trace.close()
        (run / 'stack-profile.json').write_text(json.dumps(report, indent=2) + '\n')
    if error:
        raise SystemExit(error)


if __name__ == '__main__':
    main()
