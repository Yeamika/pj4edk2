"""Compile the production UEFI file adapter against protocol fault injection."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

class SettingsFileTest(unittest.TestCase):
    def test_display_mode_lease(self):
        self.compile_and_run('display_mode.c')
    def test_fastboot_protocol(self):
        self.compile_and_run('fastboot_protocol.c')
    def test_discovery(self):
        self.compile_and_run('discovery_file.c')

    def test_standard_filesystem_failures(self):
        self.compile_and_run('settings_file.c')

    def compile_and_run(self, source):
        edk = Path(os.environ['PJB_EDK2_ROOT'])
        with tempfile.TemporaryDirectory() as output:
            exe = Path(output) / 'settings-file'
            subprocess.run([os.environ.get('CC', 'cc'), '-std=c11', '-fshort-wchar',
                '-fno-builtin', '-Werror', '-DEFIAPI=__attribute__((ms_abi))',
                '-I' + str(edk / 'MdePkg/Include'), '-I' + str(edk / 'MdePkg/Include/X64'),
                '-I' + str(Path(__file__).resolve().parents[2]/'pocketjsforedk2/include'),
                str(Path(__file__).with_name(source)), '-o', str(exe)], check=True)
            subprocess.run([str(exe)], check=True)

if __name__ == '__main__':
    unittest.main()
