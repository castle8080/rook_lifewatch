#!/usr/bin/env python3
#
# Run a small web servet to help access content.
#
import sys
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from functools import partial

HOST = "0.0.0.0"
PORT = 8080

root = "var/images" 

handler = partial(SimpleHTTPRequestHandler, directory=root)
server = ThreadingHTTPServer((HOST, PORT), handler)

print(f"Serving {root} on http://{HOST}:{PORT} (Ctrl-C to stop)")
server.serve_forever()
