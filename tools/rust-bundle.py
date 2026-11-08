#!/usr/bin/env python3
"""Identify a remotely built Rust archive and its exact source inputs."""
import argparse
import hashlib
import json
from pathlib import Path
import runpy
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sources(repo):
    result = {}
    for name in ('pocketjsforedk2', 'default-ui', 'reference'):
        for p in sorted((repo/name).rglob('*')):
            if (p.is_file() and p.suffix not in ('.md', '.py')
                    and not any(part in ('.git', 'target', '__pycache__') for part in p.parts)):
                result[str(p.relative_to(repo))] = sha(p)
    result['tools/ui-input.rs'] = sha(repo/'tools/ui-input.rs')
    return result


def identity(repo, default_ui=None):
    api = runpy.run_path(str(repo/'tools/ui-input.py'))
    return {'schema': 2, 'sources': sources(repo),
            'default_ui': api['ui_identity'](repo, default_ui)}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('action', choices=('create', 'verify'))
    p.add_argument('repo', type=Path)
    p.add_argument('bundle', type=Path)
    p.add_argument('--default-ui', type=Path, help='UI directory containing ui.json; defaults to repo/default-ui')
    a = p.parse_args()
    files = {'Rust.a': sha(a.bundle/'Rust.a')}
    files.update({str(f.relative_to(a.bundle)): sha(f)
                  for f in sorted((a.bundle/'quickjs').rglob('*')) if f.is_file()})
    record = dict(identity(a.repo, a.default_ui), files=files)
    manifest = a.bundle/'rust-build.json'
    if a.action == 'create':
        record['rustc'] = subprocess.check_output(['rustc', '--version'], text=True).strip()
        manifest.write_text(json.dumps(record, indent=2) + '\n')
    else:
        existing = json.loads(manifest.read_text())
        for name, value in record.items():
            if existing.get(name) != value:
                raise SystemExit(f'Rust bundle {name} differs; rebuild with the current pj4edk2 tree and selected UI')
        if existing.get('rustc') != 'rustc 1.93.0 (254b59607 2026-01-19)':
            raise SystemExit('Rust bundle compiler does not match the pinned version')
    print(f'Rust bundle {a.action}: passed')


if __name__ == '__main__':
    main()
