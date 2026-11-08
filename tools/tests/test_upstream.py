"""Exercise real Git selection, moving branches, replay and source consumption."""
import json
import os
from pathlib import Path
import runpy
import subprocess
import tempfile
import unittest
from unittest.mock import patch

REPO = Path(__file__).resolve().parents[2]
API = runpy.run_path(str(REPO/'tools/upstream.py'))


class UpstreamTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.cache = self.root/'cache with spaces'
        self.remotes = {}
        self.lock = {'schema': 1, 'dependencies': {}}
        for name in API['ALL_NAMES']:
            remote = self.root/name
            remote.mkdir()
            self.git(remote, 'init', '--quiet', '-b', 'main')
            self.git(remote, 'config', 'user.name', 'Source selection test')
            self.git(remote, 'config', 'user.email', 'sources@example.invalid')
            (remote/'marker').write_text('pinned')
            if name == 'edk2':
                (remote/'BaseTools').mkdir()
                (remote/'BaseTools/Makefile').write_text('all:\n\tmkdir -p bin\n\techo built > bin/fixture\n')
                (remote/'MdePkg').mkdir()
                (remote/'MdePkg/MdePkg.dec').write_text('generic EDK II fixture')
                (remote/'edksetup.sh').write_text('# fixture')
            if name == 'pocketjs':
                (remote/'Cargo.toml').write_text('[workspace]\nmembers = ["engine/core"]\n')
                (remote/'engine/core/src').mkdir(parents=True)
                (remote/'engine/core/Cargo.toml').write_text(
                    '[package]\nname="pocketjs-core"\nversion="0.1.0"\n')
                (remote/'engine/core/src/lib.rs').write_text('pub const SELECTED: &str = "pinned";\n')
            commit = self.commit(remote)
            self.git(remote, 'tag', 'tested-version')
            self.remotes[name] = remote
            self.lock['dependencies'][name] = {'repository': str(remote), 'commit': commit}
        self.lock_file = self.root/'pinned.json'
        self.write_lock()

    def git(self, root, *args):
        return subprocess.check_output(['git', '-C', str(root), *args], text=True,
                                       stderr=subprocess.PIPE).strip()

    def commit(self, root):
        self.git(root, 'add', '--all')
        self.git(root, 'commit', '--quiet', '-m', 'fixture revision')
        return self.git(root, 'rev-parse', 'HEAD')

    def write_lock(self):
        self.lock_file.write_text(json.dumps(self.lock))

    def resolve(self, refs=(), repos=(), lock=None, **kwargs):
        return API['resolve'](REPO, self.cache, lock or self.lock_file, repos, refs, **kwargs)

    def test_rust_resolution_does_not_fetch_or_depend_on_edk2(self):
        self.remotes['edk2'].rename(self.root/'edk2-offline')
        record, paths = self.resolve()
        self.assertNotIn('edk2', paths)
        self.assertNotIn('edk2', record['dependencies'])
        full_lock = json.loads(json.dumps(record))
        full_lock['dependencies']['edk2'] = {'repository': 'unused', 'commit': 'a'*40}
        self.assertEqual(API['identity'](record), API['identity'](full_lock))
        with self.assertRaisesRegex(ValueError, 'Rust stage does not use EDK II'):
            self.resolve(refs=['edk2=latest'])

    def test_generic_edk2_download_replay_and_private_basetools(self):
        record, paths = self.resolve(with_edk2=True)
        self.assertEqual(record['dependencies']['edk2']['commit'],
                         self.lock['dependencies']['edk2']['commit'])
        output = self.root/'private tools'
        subprocess.run(['python3', str(REPO/'tools/build-basetools.py'), '--edk2',
                        paths['edk2'], '--output', str(output)], check=True, stdout=subprocess.PIPE)
        self.assertEqual((output/'bin/fixture').read_text().strip(), 'built')
        self.assertFalse((Path(paths['edk2'])/'BaseTools/bin').exists())
        replay = self.root/'with-edk2.json'
        replay.write_text(json.dumps(record))
        self.remotes['edk2'].rename(self.root/'edk2-offline')
        repeated, _ = self.resolve(with_edk2=True, lock=replay)
        self.assertEqual(record['dependencies']['edk2'], repeated['dependencies']['edk2'])

    def test_external_edk2_accepts_caller_revision_without_fetch_or_default_pin(self):
        edk2 = self.remotes['edk2']
        (edk2/'marker').write_text('another platform revision')
        head = self.commit(edk2)
        self.assertNotEqual(head, self.lock['dependencies']['edk2']['commit'])
        record, paths = self.resolve(edk2=edk2)
        self.assertEqual(record['dependencies']['edk2']['commit'], head)
        self.assertEqual(Path(paths['edk2']), edk2)
        with self.assertRaisesRegex(ValueError, 'either --edk2 or an upstream edk2 selection'):
            self.resolve(edk2=edk2, refs=['edk2=latest'])

    def test_edk2_repository_and_latest_are_selectable(self):
        edk2 = self.remotes['edk2']
        (edk2/'marker').write_text('new generic EDK II')
        head = self.commit(edk2)
        record, _ = self.resolve(with_edk2=True, refs=['edk2=latest'], repos=['edk2='+str(edk2)])
        self.assertEqual(record['dependencies']['edk2']['commit'], head)

    def test_edk2_submodules_follow_gitlinks_and_changed_cache_is_rejected(self):
        brotli = self.root/'brotli'
        brotli.mkdir()
        self.git(brotli, 'init', '--quiet', '-b', 'main')
        self.git(brotli, 'config', 'user.name', 'Source selection test')
        self.git(brotli, 'config', 'user.email', 'sources@example.invalid')
        header = brotli/'c/include/brotli/encode.h'
        header.parent.mkdir(parents=True)
        header.write_text('pinned Brotli header')
        commit = self.commit(brotli)
        edk2 = self.remotes['edk2']
        for path in API['BROTLI_PATHS']:
            self.git(edk2, '-c', 'protocol.file.allow=always', 'submodule', 'add', '--quiet',
                     brotli.as_uri(), path)
        self.lock['dependencies']['edk2']['commit'] = self.commit(edk2)
        self.write_lock()
        header.write_text('new upstream header')
        self.commit(brotli)
        with patch.dict(os.environ, {'GIT_ALLOW_PROTOCOL': 'file'}):
            record, paths = self.resolve(with_edk2=True)
        self.assertEqual(record['dependencies']['edk2']['submodules'],
                         {path: commit for path in API['BROTLI_PATHS']})
        for path in API['BROTLI_PATHS']:
            self.assertEqual(self.git(Path(paths['edk2'])/path, 'rev-parse', '--is-shallow-repository'), 'true')
        downloaded = Path(paths['edk2'])/API['BROTLI']/'c/include/brotli/encode.h'
        self.assertEqual(downloaded.read_text(), 'pinned Brotli header')
        downloaded.write_text('changed cached submodule')
        with self.assertRaisesRegex(ValueError, 'cached source was changed'):
            self.resolve(with_edk2=True)

    def test_pin_tag_and_selected_sources_reach_build_tree(self):
        original = (REPO/'vendor/pocketjs/engine/core/src/lib.rs').read_bytes()
        record, paths = self.resolve(refs=['pocketjs=tested-version'])
        self.assertEqual(record['dependencies']['pocketjs']['commit'],
                         self.lock['dependencies']['pocketjs']['commit'])
        tree = API['prepare'](REPO, self.root/'build', record, paths)
        self.assertIn('SELECTED', (tree/'vendor/pocketjs/engine/core/src/lib.rs').read_text())
        self.assertEqual(original, (REPO/'vendor/pocketjs/engine/core/src/lib.rs').read_bytes())

    def test_latest_refreshes_but_saved_lock_replays_offline(self):
        first, _ = self.resolve(refs=['pocketjs=latest'])
        remote = self.remotes['pocketjs']
        (remote/'marker').write_text('new head')
        head = self.commit(remote)
        pinned, _ = self.resolve()
        self.assertEqual(pinned['dependencies']['pocketjs']['commit'],
                         first['dependencies']['pocketjs']['commit'])
        latest, _ = self.resolve(refs=['pocketjs=latest'])
        self.assertEqual(latest['dependencies']['pocketjs']['commit'], head)
        replay = self.root/'resolved.json'
        replay.write_text(json.dumps(latest))
        for name, remote in self.remotes.items():
            remote.rename(remote.with_name(name+'-offline'))
        repeated, _ = self.resolve(lock=replay)
        self.assertEqual(API['identity'](latest), API['identity'](repeated))

    def test_repository_and_branch_can_be_selected(self):
        fork = self.root/'fork'
        subprocess.run(['git', 'clone', '--quiet', str(self.remotes['pocketjs']), str(fork)], check=True)
        self.git(fork, 'config', 'user.name', 'Source selection test')
        self.git(fork, 'config', 'user.email', 'sources@example.invalid')
        self.git(fork, 'checkout', '--quiet', '-b', 'integration')
        (fork/'marker').write_text('fork implementation')
        head = self.commit(fork)
        record, paths = self.resolve(refs=['pocketjs=integration'], repos=['pocketjs='+str(fork)])
        self.assertEqual(record['dependencies']['pocketjs']['commit'], head)
        self.assertEqual((Path(paths['pocketjs'])/'marker').read_text(), 'fork implementation')

    def test_changed_cache_and_missing_revision_are_rejected(self):
        _, paths = self.resolve()
        (Path(paths['quickjs'])/'marker').write_text('unrecorded modification')
        with self.assertRaisesRegex(ValueError, 'cached source was changed'):
            self.resolve()
        with self.assertRaises(subprocess.CalledProcessError):
            self.resolve(refs=['edk2-libc=no-such-branch'])


if __name__ == '__main__':
    unittest.main()
