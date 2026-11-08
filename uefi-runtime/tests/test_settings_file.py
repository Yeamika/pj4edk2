"""Compile the production UEFI file adapter against protocol fault injection."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

class SettingsFileTest(unittest.TestCase):
    def test_standard_filesystem_failures(self):
        edk = Path(os.environ['PJB_EDK2_ROOT'])
        with tempfile.TemporaryDirectory() as output:
            exe = Path(output) / 'settings-file'
            subprocess.run([os.environ.get('CC', 'cc'), '-std=c11', '-fshort-wchar',
                '-fno-builtin', '-Werror', '-DEFIAPI=__attribute__((ms_abi))',
                '-I' + str(edk / 'MdePkg/Include'), '-I' + str(edk / 'MdePkg/Include/X64'),
                str(Path(__file__).with_name('settings_file.c')), '-o', str(exe)], check=True)
            subprocess.run([str(exe)], check=True)

if __name__ == '__main__':
    unittest.main()
