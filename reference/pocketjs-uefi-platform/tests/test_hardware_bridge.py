"""Run the real optional charge/power bridge and child launch boundary."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

class HardwareBridgeTest(unittest.TestCase):
    def test_bounded_boot_log_without_serial(self):
        edk=Path(os.environ['PJB_EDK2_ROOT'])
        with tempfile.TemporaryDirectory() as output:
            exe=Path(output)/'boot-log'
            subprocess.run([os.environ.get('CC','cc'),'-std=c11','-fshort-wchar',
                '-fno-builtin','-Werror','-DEFIAPI=__attribute__((ms_abi))',
                '-ffunction-sections','-fdata-sections','-Wl,--gc-sections',
                '-I'+str(edk/'MdePkg/Include'),'-I'+str(edk/'MdePkg/Include/X64'),
                '-I'+str(edk/'ShellPkg/Include'),
                str(Path(__file__).with_name('boot_log.c')),'-o',str(exe)],check=True)
            subprocess.run([str(exe)],check=True)

    def test_optional_protocol_and_failure_state(self):
        edk=Path(os.environ['PJB_EDK2_ROOT'])
        abi=Path(__file__).resolve().parents[3]/'pocketjsforedk2/include'
        with tempfile.TemporaryDirectory() as output:
            exe=Path(output)/'hardware-bridge'
            subprocess.run([os.environ.get('CC','cc'),'-std=c11','-fshort-wchar',
                '-fno-builtin','-Werror','-DEFIAPI=__attribute__((ms_abi))',
                '-ffunction-sections','-fdata-sections','-Wl,--gc-sections',
                '-I'+str(edk/'MdePkg/Include'),'-I'+str(edk/'MdePkg/Include/X64'),
                '-I'+str(abi),str(Path(__file__).with_name('hardware_bridge.c')),'-o',str(exe)],check=True)
            subprocess.run([str(exe)],check=True)

if __name__=='__main__':unittest.main()
