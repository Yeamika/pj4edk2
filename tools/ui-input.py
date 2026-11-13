#!/usr/bin/env python3
"""Validate and fingerprint the files embedded from a default UI directory."""
import argparse
import hashlib
import json
from pathlib import Path


def ui_identity(repo, default_ui=None):
    root = Path(default_ui if default_ui is not None else repo/'default-ui').resolve(strict=True)
    manifest = root/'ui.json'
    if not manifest.resolve(strict=True).is_relative_to(root):
        raise ValueError('ui.json must stay inside the UI directory')
    config = json.loads(manifest.read_text(encoding='utf-8'))
    if not isinstance(config, dict) or type(config.get('schema')) is not int or config['schema'] != 1:
        raise ValueError('unsupported UI schema; expected ui.json schema 1')
    scripts = config.get('scripts')
    if not isinstance(scripts, list) or not scripts:
        raise ValueError('UI scripts must be a nonempty ordered array')
    files = {'ui.json': hashlib.sha256(manifest.read_bytes()).hexdigest()}

    def record(name):
        if (not isinstance(name, str) or not name
                or any(c in name for c in ('\\', '\n', '\r', '\0'))
                or any(part in ('', '.', '..') for part in name.split('/'))):
            raise ValueError(f'UI file paths must stay inside the UI directory: {name!r}')
        path = (root/name).resolve(strict=True)
        if not path.is_relative_to(root) or not path.is_file():
            raise ValueError(f'UI input must be a file inside the UI directory: {name}')
        files[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        return path

    for name in scripts:
        record(name).read_text(encoding='utf-8')
    if len(set(scripts)) != len(scripts):
        raise ValueError('duplicate UI script')
    font = config.get('font')
    record(font)
    if 'font_license' in config:
        record(config['font_license'])
    return {'schema': 1, 'scripts': scripts, 'font': font, 'files': files}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('directory', type=Path)
    a = p.parse_args()
    try:
        ui_identity(Path(__file__).resolve().parent.parent, a.directory)
    except (ValueError, OSError) as e:
        p.error(str(e))


if __name__ == '__main__':
    main()
