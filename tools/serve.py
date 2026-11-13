#!/usr/bin/env python3
"""Serve the browser-only viewer; no build or external packages required."""
import argparse
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

class PreviewHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cache-Control', 'no-store')
        super().end_headers()

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--port',type=int,default=1299)
p.add_argument('--bind',default='127.0.0.1')
a=p.parse_args()
root=Path(__file__).resolve().parents[1]
print(f'Viewer: http://{a.bind}:{a.port}/',flush=True)
ThreadingHTTPServer((a.bind,a.port),partial(PreviewHandler,directory=str(root))).serve_forever()
