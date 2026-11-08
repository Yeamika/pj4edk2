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
    for name in ('pocketjsforedk2', 'default-ui', 'uefi-runtime', 'crates', 'tools'):
        for p in sorted((repo/name).rglob('*')):
            if (p.is_file() and p.suffix != '.md'
                    and not any(part in ('.git', 'target', '__pycache__') for part in p.relative_to(repo).parts)):
                result[str(p.relative_to(repo))] = sha(p)
    for name in ('rust-toolchain.toml', 'sources.lock.json', 'build.sh', 'build-rust.sh'):
        result[name] = sha(repo/name)
    return result


def identity(repo, default_ui=None, source_lock=None):
    api = runpy.run_path(str(repo/'tools/ui-input.py'))
    upstream = runpy.run_path(str(repo/'tools/upstream.py'))
    lock = json.loads(Path(source_lock or repo/'sources.lock.json').read_text())
    return {'schema': 3, 'sources': sources(repo), 'upstream': upstream['identity'](lock),
            'default_ui': api['ui_identity'](repo, default_ui)}


def tree_files(root):
    return {str(f.relative_to(root)): sha(f) for f in sorted(root.rglob('*'))
            if f.is_file() and '.git' not in f.relative_to(root).parts}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('action', choices=('create', 'verify'))
    p.add_argument('repo', type=Path)
    p.add_argument('bundle', type=Path)
    p.add_argument('--default-ui', type=Path, help='UI directory containing ui.json; defaults to repo/default-ui')
    p.add_argument('--source-lock', type=Path, help='resolved upstream revisions selected for this build')
    p.add_argument('--quickjs', type=Path, help='verify bundle QuickJS files against the selected Git source')
    a = p.parse_args()
    files = {name: sha(a.bundle/name) for name in ('Rust.a', 'Cargo.lock', 'upstream.lock.json')}
    quickjs = tree_files(a.bundle/'quickjs')
    files.update({'quickjs/'+name: digest for name, digest in quickjs.items()})
    if a.quickjs and quickjs != tree_files(a.quickjs):
        raise SystemExit('Rust bundle QuickJS source differs from the selected upstream revision')
    record = dict(identity(a.repo, a.default_ui, a.source_lock), files=files)
    upstream_api = runpy.run_path(str(a.repo/'tools/upstream.py'))
    if upstream_api['identity'](json.loads((a.bundle/'upstream.lock.json').read_text())) != record['upstream']:
        raise SystemExit('Rust bundle upstream selection differs')
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
