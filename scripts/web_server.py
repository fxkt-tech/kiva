#!/usr/bin/env python3
"""Local web server and fixed-upstream LLM proxy (Python standard library only)."""

import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import urllib.error
import urllib.request


class Handler(SimpleHTTPRequestHandler):
    def send_json(self, status, value):
        body = json.dumps(value, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/api/config":
            self.send_json(200, {"enabled": self.server.llm_enabled})
        elif self.path.startswith("/api/"):
            self.send_json(404, {"error": "Unknown API route"})
        else:
            super().do_GET()

    def do_POST(self):
        if self.path != "/api/chat/completions":
            self.send_json(404, {"error": "Unknown API route"})
            return
        if not self.server.llm_enabled:
            self.send_json(503, {"error": "LLM is not configured"})
            return
        # Reject cross-origin browser requests. Trunk forwards the browser's Origin.
        origin = self.headers.get("Origin")
        if origin and origin not in {"http://localhost:8080", "http://127.0.0.1:8080",
                                     f"http://localhost:{self.server.server_port}",
                                     f"http://127.0.0.1:{self.server.server_port}"}:
            self.send_json(403, {"error": "Origin is not allowed"})
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            if not 0 < length <= 1_048_576:
                self.send_json(413, {"error": "Invalid request size"})
                return
            body = json.loads(self.rfile.read(length))
            if not isinstance(body, dict) or not isinstance(body.get("messages"), list):
                raise ValueError("Expected chat messages")
            body["model"] = self.server.llm_model
            body["stream"] = False
        except (ValueError, UnicodeDecodeError):
            self.send_json(400, {"error": "Invalid chat request"})
            return
        request = urllib.request.Request(
            self.server.llm_base_url.rstrip("/") + "/chat/completions",
            data=json.dumps(body).encode("utf-8"),
            headers={"Content-Type": "application/json",
                     "Authorization": "Bearer " + self.server.llm_api_key},
            method="POST",
        )
        try:
            with urllib.request.urlopen(request, timeout=90) as response:
                result = json.load(response)
            self.send_json(200, result)
        except urllib.error.HTTPError as error:
            # Do not return upstream error bodies, which may contain credentials.
            self.send_json(502, {"error": f"LLM upstream returned HTTP {error.code}"})
        except (OSError, ValueError, urllib.error.URLError):
            self.send_json(502, {"error": "LLM upstream request failed"})


def make_server(port, directory):
    server = ThreadingHTTPServer(("127.0.0.1", port), partial(Handler, directory=str(directory)))
    server.llm_base_url = os.environ.get("KIVA_LLM_BASE_URL", "").strip()
    server.llm_api_key = os.environ.get("KIVA_LLM_API_KEY", "").strip()
    server.llm_model = os.environ.get("KIVA_LLM_MODEL", "").strip()
    server.llm_enabled = bool(server.llm_base_url and server.llm_api_key and server.llm_model)
    return server


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--directory", type=Path, default=Path(__file__).resolve().parent.parent / "dist")
    args = parser.parse_args()
    with make_server(args.port, args.directory) as server:
        mode = "LLM configured" if server.llm_enabled else "offline rule-based AI"
        print(f"Kiva: http://localhost:{args.port} ({mode})", flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
