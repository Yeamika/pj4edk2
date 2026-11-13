"""Validate both GPT copies and the ESP bounds without touching a real disk."""
import importlib.util,struct,tempfile,unittest,zlib
from pathlib import Path
spec=importlib.util.spec_from_file_location('disk',Path(__file__).with_name('disk.py'))
disk=importlib.util.module_from_spec(spec);spec.loader.exec_module(disk)
class DiskTest(unittest.TestCase):
    def test_gpt_crc_bounds_and_backup(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);esp=root/'esp';esp.write_bytes(b'fat-fixture')
            output=root/'disk.img';disk.write_disk(output,esp,4);data=output.read_bytes()
            for lba in [1,len(data)//512-1]:
                h=bytearray(data[lba*512:(lba+1)*512]);self.assertEqual(h[:8],b'EFI PART')
                crc=struct.unpack_from('<I',h,16)[0];struct.pack_into('<I',h,16,0)
                self.assertEqual(crc,zlib.crc32(h[:92]))
                table=struct.unpack_from('<Q',h,72)[0]*512;entries=data[table:table+128*128]
                self.assertEqual(zlib.crc32(entries),struct.unpack_from('<I',h,88)[0])
                self.assertEqual(entries[:16],disk.ESP_TYPE.bytes_le)
                self.assertEqual(struct.unpack_from('<Q',entries,32)[0],2048)
            self.assertEqual(data[2048*512:2048*512+len(b'fat-fixture')],b'fat-fixture')
            with self.assertRaises(FileExistsError):disk.write_disk(output,esp,4)
if __name__=='__main__':unittest.main()
