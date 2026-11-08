#!/usr/bin/env python3
"""Resolve upstream Git revisions and prepare an isolated runtime source tree."""
import argparse
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

NAMES = ('edk2-libc', 'pocketjs', 'quickjs')
POCKETJS_MEMBERS = (
    'devices/gles/pocket-gles', 'devices/gles/pocket-gles1', 'devices/gles/pocket-gles2',
    'engine/core', 'engine/drivers', 'engine/modules/media', 'engine/package',
    'engine/present', 'engine/raster', 'engine/render/draw-ops',
    'engine/render/glyph-pages', 'engine/render/tex-cache', 'engine/render/tri-batch',
    'engine/runtime', 'engine/runtime-cabi', 'engine/guest-rquickjs', 'engine/spec',
)


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def overrides(values):
    result = {}
    for item in values:
        name, separator, value = item.partition('=')
        if name not in NAMES or not separator or not value or name in result:
            raise ValueError(f'Expected one NAME=VALUE for each of: {", ".join(NAMES)}')
        if value.startswith('-') or '\n' in value or '\r' in value:
            raise ValueError('Invalid source repository or revision')
        result[name] = value
    return result


def selections(repo, lock=None, source_repos=(), source_refs=()):
    defaults = json.loads((repo/'sources.lock.json').read_text())['dependencies']
    supplied = json.loads(Path(lock).read_text())['dependencies'] if lock else defaults
    repos, refs = overrides(source_repos), overrides(source_refs)
    result = {}
    for name in NAMES:
        value = supplied[name]
        repository = repos.get(name, value['repository'])
        ref = refs.get(name, value.get('ref', value.get('commit')))
        if not isinstance(ref, str) or not ref or ref.startswith('-'):
            raise ValueError(f'{name}: supply a commit, branch, tag or latest')
        if not isinstance(repository, str) or not repository or repository.startswith('-'):
            raise ValueError(f'{name}: invalid repository')
        if repository.startswith(('.', '/', '~')):
            repository = str(Path(repository).expanduser().resolve())
        result[name] = {'repository': repository, 'ref': ref,
                        'requested_ref': refs.get(name, value.get('requested_ref', ref))}
    return result


def identity(lock):
    dependencies = lock['dependencies']
    result = {}
    for name in NAMES:
        entry = dependencies[name]
        if not re.fullmatch(r'[0-9a-f]{40}', entry['commit']):
            raise ValueError(f'{name}: source lock must contain a resolved full commit')
        result[name] = {key: entry[key] for key in ('repository', 'commit')}
    return result


def outside_sources(repo, path):
    path = path.resolve()
    if path == repo or repo in path.parents:
        raise ValueError('Upstream output/cache must be outside the pj4edk2 source tree')
    if os.environ.get('PJB_DEFAULT_UI'):
        ui = Path(os.environ['PJB_DEFAULT_UI']).resolve()
        if path == ui or ui in path.parents:
            raise ValueError('Upstream output/cache must be outside the UI directory')
    return path


@contextlib.contextmanager
def cache_lock(cache):
    cache.mkdir(parents=True, exist_ok=True)
    with (cache/'.lock').open('a') as stream:
        fcntl.flock(stream, fcntl.LOCK_EX)
        yield


def checkout(cache, name, selected):
    repository, ref = selected['repository'], selected['ref']
    key = hashlib.sha256(repository.encode()).hexdigest()
    database = cache/'git'/key
    database.parent.mkdir(parents=True, exist_ok=True)
    if not database.exists():
        subprocess.run(['git', 'init', '--quiet', '--bare', str(database)], check=True)
    # Mutable references are resolved on every invocation. A saved lock reuses
    # the exact object without contacting a moving branch again.
    pinned = bool(re.fullmatch(r'[0-9a-f]{40}', ref))
    available = pinned and subprocess.run(
        ['git', '-C', str(database), 'cat-file', '-e', ref+'^{commit}'],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0
    if not available:
        print(f'Fetching {name}: {ref}', flush=True)
        subprocess.run(['git', '-C', str(database), 'fetch', '--quiet', '--depth=1',
                        '--no-tags', '--', repository, 'HEAD' if ref == 'latest' else ref], check=True)
    commit = git(database, 'rev-parse', (ref if available else 'FETCH_HEAD')+'^{commit}')
    if pinned and commit != ref:
        raise ValueError(f'{name}: fetched commit does not match requested commit')
    reference = 'refs/heads/pinned-'+commit
    git(database, 'update-ref', reference, commit)
    git(database, 'symbolic-ref', 'HEAD', reference)
    destination = cache/'checkouts'/key/commit
    if destination.exists():
        if (git(destination, 'rev-parse', 'HEAD') != commit
                or git(destination, 'status', '--porcelain', '--untracked-files=all')):
            raise ValueError(f'{name}: cached source was changed: {destination}; use a fresh cache')
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        temporary = destination.with_name(commit+f'.tmp-{os.getpid()}')
        try:
            subprocess.run(['git', 'clone', '--quiet', '--shared', '--no-checkout',
                            str(database), str(temporary)], check=True)
            subprocess.run(['git', '-C', str(temporary), 'checkout', '--quiet', '--detach', commit], check=True)
            temporary.rename(destination)
        finally:
            if temporary.exists():
                shutil.rmtree(temporary)
    return {'repository': repository, 'commit': commit,
            'requested_ref': selected['requested_ref'],
            'tree': git(database, 'rev-parse', commit+'^{tree}')}, destination


def resolve(repo, cache, lock=None, source_repos=(), source_refs=()):
    selected = selections(repo, lock, source_repos, source_refs)
    record, paths = {'schema': 1, 'dependencies': {}}, {}
    with cache_lock(cache):
        for name, value in selected.items():
            record['dependencies'][name], path = checkout(cache, name, value)
            paths[name] = str(path)
    return record, paths


def copy_sources(source, destination):
    shutil.copytree(source, destination, symlinks=True, ignore=shutil.ignore_patterns(
        '.git', 'target', '__pycache__', '*.pyc', '.venv-docs'))


def prepare(repo, output, record, paths):
    tree = output/'source-tree'
    if tree.exists():
        raise ValueError('Retain existing prepared sources; choose a fresh build directory')
    tree.mkdir(parents=True)
    for name in ('pocketjsforedk2', 'default-ui', 'uefi-runtime', 'crates', 'tools'):
        copy_sources(repo/name, tree/name)
    shutil.copy2(repo/'rust-toolchain.toml', tree/'rust-toolchain.toml')
    pocketjs = tree/'vendor/pocketjs'
    copy_sources(Path(paths['pocketjs']), pocketjs)
    # Only the firmware graph participates in this integration. Preserve the
    # selected upstream's package metadata and dependencies; unrelated desktop
    # and device programs must not expand Cargo's lock graph.
    manifest = pocketjs/'Cargo.toml'
    members = [name for name in POCKETJS_MEMBERS if (pocketjs/name/'Cargo.toml').is_file()]
    text, count = re.subn(r'(?m)^members\s*=\s*\[[^\]]*\]',
                          'members = '+json.dumps(members), manifest.read_text(), count=1)
    if count != 1:
        raise ValueError('Selected PocketJS has no supported workspace members list')
    manifest.write_text(text)
    write_json(output/'upstream.lock.json', record)
    write_json(output/'source-paths.json', dict(paths, source_tree=str(tree)))
    return tree


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('action', choices=('resolve', 'prepare'))
    p.add_argument('--repo', type=Path, default=Path(__file__).resolve().parents[1])
    p.add_argument('--output', type=Path, required=True, help='lock file for resolve; build directory for prepare')
    p.add_argument('--source-cache', type=Path)
    p.add_argument('--source-lock', type=Path, help='replay a resolved upstream.lock.json')
    p.add_argument('--source-repo', action='append', default=[], metavar='NAME=URL')
    p.add_argument('--source-ref', action='append', default=[], metavar='NAME=REV',
                   help='full commit, branch, tag or latest (remote default branch HEAD)')
    return p


def main():
    a = parser().parse_args()
    repo, output = a.repo.resolve(), a.output.resolve()
    outside_sources(repo, output)
    cache = outside_sources(repo, a.source_cache or Path(os.environ.get('PJB_SOURCE_CACHE') or
                            str((output if a.action == 'prepare' else output.parent)/'upstream-cache')))
    record, paths = resolve(repo, cache, a.source_lock, a.source_repo, a.source_ref)
    if a.action == 'resolve':
        write_json(output, record)
    else:
        prepare(repo, output, record, paths)
    print(json.dumps(identity(record), indent=2))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
