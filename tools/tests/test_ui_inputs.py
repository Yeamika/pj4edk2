"""UI input identity and optional real Cargo cache integration checks."""
import json
import os
from pathlib import Path
import runpy
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[2]
API = runpy.run_path(str(REPO/'tools/rust-bundle.py'))


class UiInputsTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.ui = self.root/'external UI'
        self.ui.mkdir()
        (self.ui/'one.js').write_text('globalThis.first = 1;')
        (self.ui/'two.js').write_text('globalThis.second = 2;')
        (self.ui/'font.ttf').write_bytes((REPO/'default-ui/assets/Inter-Regular.ttf').read_bytes())
        self.config = {'schema': 1, 'scripts': ['two.js', 'one.js'], 'font': 'font.ttf'}
        self.write_config()

    def write_config(self):
        (self.ui/'ui.json').write_text(json.dumps(self.config))

    def identity(self, directory=None):
        return API['identity'](REPO, directory or self.ui)['default_ui']

    def test_relocation_and_order(self):
        copied = self.root/'relocated'
        shutil.copytree(self.ui, copied)
        original = self.identity()
        self.assertEqual(original, self.identity(copied))
        self.config['scripts'].reverse()
        self.write_config()
        self.assertNotEqual(original, self.identity())

    def test_script_and_font_changes(self):
        first = self.identity()
        (self.ui/'one.js').write_text('globalThis.first = 99;')
        second = self.identity()
        self.assertNotEqual(first, second)
        (self.ui/'font.ttf').write_bytes(b'different-font')
        self.assertNotEqual(second, self.identity())

    def test_missing_and_invalid_inputs(self):
        for scripts in ([], ['missing.js'], ['one.js', 'one.js'], ['../one.js'], ['/one.js'], [42]):
            with self.subTest(scripts=scripts):
                self.config['scripts'] = scripts
                self.write_config()
                with self.assertRaises((ValueError, OSError)):
                    self.identity()

    def test_symlink_cannot_escape(self):
        (self.root/'outside.js').write_text('outside')
        (self.ui/'link.js').symlink_to(self.root/'outside.js')
        self.config['scripts'] = ['link.js']
        self.write_config()
        with self.assertRaises(ValueError):
            self.identity()

    def test_bundle_rejects_different_ui(self):
        bundle = self.root/'bundle'
        bundle.mkdir()
        (bundle/'Rust.a').write_bytes(b'test-archive')
        (bundle/'Cargo.lock').write_text('test-lock')
        (bundle/'upstream.lock.json').write_bytes((REPO/'sources.lock.json').read_bytes())
        record = dict(API['identity'](REPO, self.ui),
                      files={name: API['sha'](bundle/name) for name in
                             ('Rust.a', 'Cargo.lock', 'upstream.lock.json')},
                      rustc='rustc 1.93.0 (254b59607 2026-01-19)')
        (bundle/'rust-build.json').write_text(json.dumps(record))
        command = [sys.executable, str(REPO/'tools/rust-bundle.py'), 'verify', str(REPO),
                   str(bundle), '--default-ui', str(self.ui)]
        subprocess.run(command, check=True, capture_output=True, text=True)
        (self.ui/'one.js').write_text('changed after building')
        failed = subprocess.run(command, capture_output=True, text=True)
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn('default_ui differs', failed.stderr)

    def test_bundle_rejects_different_upstream(self):
        bundle = self.root/'bundle'
        bundle.mkdir()
        for name in ('Rust.a', 'Cargo.lock'):
            (bundle/name).write_bytes(b'fixture')
        (bundle/'upstream.lock.json').write_bytes((REPO/'sources.lock.json').read_bytes())
        record = dict(API['identity'](REPO, self.ui),
                      files={name: API['sha'](bundle/name) for name in
                             ('Rust.a', 'Cargo.lock', 'upstream.lock.json')},
                      rustc='rustc 1.93.0 (254b59607 2026-01-19)')
        (bundle/'rust-build.json').write_text(json.dumps(record))
        selection = json.loads((bundle/'upstream.lock.json').read_text())
        selection['dependencies']['pocketjs']['commit'] = 'f'*40
        selected = self.root/'another-upstream.json'
        selected.write_text(json.dumps(selection))
        result = subprocess.run([sys.executable, str(REPO/'tools/rust-bundle.py'), 'verify',
                                 str(REPO), str(bundle), '--default-ui', str(self.ui),
                                 '--source-lock', str(selected)], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('upstream selection differs', result.stderr)

    @unittest.skipUnless(os.environ.get('PJB_RUN_CARGO_TESTS') == '1', 'set PJB_RUN_CARGO_TESTS=1 for Cargo builds')
    def test_cargo_rebuilds_scripts_font_and_directory_selection(self):
        consumer = self.root/'consumer'
        (consumer/'src').mkdir(parents=True)
        (consumer/'Cargo.toml').write_text(
            '[package]\nname="ui-input-check"\nversion="0.1.0"\nedition="2024"\n'
            '[dependencies]\npocketjs-boot-ui={path=' + json.dumps(str(REPO/'default-ui')) + '}\n'
            'pocketjs-boot-assets={path=' + json.dumps(str(REPO/'crates/boot-assets')) + '}\n')
        (consumer/'src/main.rs').write_text('''
fn main() {
    let font = pocketjs_boot_assets::atlases(false).into_iter().flatten()
        .fold(0u64, |hash, byte| hash.wrapping_mul(31).wrapping_add(*byte as u64));
    println!("font={font}");
    for script in pocketjs_boot_ui::scripts() {
        println!("{}:{}", script.name, script.source);
    }
}
''')
        env = dict(os.environ, CARGO_TARGET_DIR=str(self.root/'shared-target'), PJB_DEFAULT_UI=str(self.ui))
        command = ['cargo', 'run', '--offline', '--quiet', '--manifest-path', str(consumer/'Cargo.toml')]

        def build():
            result = subprocess.run(command, env=env, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout+result.stderr)
            return result.stdout.splitlines()

        first = build()
        self.assertEqual(first[1:], ['two.js:globalThis.second = 2;', 'one.js:globalThis.first = 1;'])
        (self.ui/'one.js').write_text('globalThis.first = 99;')
        self.assertEqual(build()[2], 'one.js:globalThis.first = 99;')
        font = bytearray((self.ui/'font.ttf').read_bytes())
        for entry in range(struct.unpack_from('>H', font, 4)[0]):
            offset = 12 + entry * 16
            if font[offset:offset+4] == b'head':
                head = struct.unpack_from('>I', font, offset+8)[0]
                units = struct.unpack_from('>H', font, head+18)[0]
                struct.pack_into('>H', font, head+18, units*2)
                break
        else:
            self.fail('fixture font has no head table')
        (self.ui/'font.ttf').write_bytes(font)
        self.assertNotEqual(first[0], build()[0])
        alternate = self.root/'another UI'
        shutil.copytree(self.ui, alternate)
        (alternate/'two.js').write_text('globalThis.alternate = true;')
        env['PJB_DEFAULT_UI'] = str(alternate)
        self.assertEqual(build()[1], 'two.js:globalThis.alternate = true;')
        (alternate/'two.js').unlink()
        failed = subprocess.run(command, env=env, capture_output=True, text=True)
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn('UI input file must exist', failed.stderr)


if __name__ == '__main__':
    unittest.main()
