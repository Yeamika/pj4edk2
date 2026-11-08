"""Exercise real Git selection, moving branches, replay and source consumption."""
import json
from pathlib import Path
import runpy
import subprocess
import tempfile
import unittest

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
        for name in API['NAMES']:
            remote = self.root/name
            remote.mkdir()
            self.git(remote, 'init', '--quiet', '-b', 'main')
            self.git(remote, 'config', 'user.name', 'Source selection test')
            self.git(remote, 'config', 'user.email', 'sources@example.invalid')
            (remote/'marker').write_text('pinned')
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

    def resolve(self, refs=(), repos=(), lock=None):
        return API['resolve'](REPO, self.cache, lock or self.lock_file, repos, refs)

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
