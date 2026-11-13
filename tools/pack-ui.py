#!/usr/bin/env python3
"""Build one upstream PocketJS .pocket UI package, including browser and firmware fonts."""
import argparse,os,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--ui',type=Path)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--target-dir',type=Path)
a=p.parse_args();repo=Path(__file__).resolve().parents[1]
ui=(a.ui or repo/'default-ui').resolve();env=os.environ.copy();env['PJB_DEFAULT_UI']=str(ui)
command=['cargo','run','--manifest-path',str(repo/'tools/ui-package/Cargo.toml')]
if a.target_dir:command+=['--target-dir',str(a.target_dir.resolve())]
command+=['--',str(ui),str(a.output.resolve())]
subprocess.run(command,env=env,check=True)
