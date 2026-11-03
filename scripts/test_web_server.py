import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from tempfile import TemporaryDirectory
import threading
import unittest
from unittest.mock import patch
import urllib.error
import urllib.request

from web_server import make_server


class WebServerTests(unittest.TestCase):
    def setUp(self):
        self.directory = TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        Path(self.directory.name, "index.html").write_text("Kiva preview")
        self.environment = patch.dict("os.environ", {
            "KIVA_LLM_BASE_URL": "", "KIVA_LLM_API_KEY": "", "KIVA_LLM_MODEL": "",
        })
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.server = self.start(make_server(0, self.directory.name))
        self.url = f"http://127.0.0.1:{self.server.server_port}"

    def start(self, server):
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(thread.join)
        self.addCleanup(server.shutdown)
        return server

    def post(self, body, **headers):
        return urllib.request.urlopen(urllib.request.Request(
            self.url + "/api/chat/completions", data=json.dumps(body).encode(),
            headers={"Content-Type": "application/json", **headers},
        ))

    def test_static_preview_and_offline_config(self):
        with urllib.request.urlopen(self.url + "/") as response:
            self.assertEqual(response.read(), b"Kiva preview")
        with urllib.request.urlopen(self.url + "/api/config") as response:
            self.assertEqual(json.load(response), {"enabled": False})
        with self.assertRaises(urllib.error.HTTPError) as error:
            self.post({"messages": []})
        self.assertEqual(error.exception.code, 503)

    def test_proxy_injects_credentials_and_model_without_exposing_them(self):
        received = {}

        class Upstream(BaseHTTPRequestHandler):
            def do_POST(self):
                received["path"] = self.path
                received["auth"] = self.headers.get("Authorization")
                received["body"] = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                body = b'{"choices":[{"message":{"content":"hello"}}]}'
                self.send_response(200)
                self.end_headers()
                self.wfile.write(body)

        upstream = self.start(ThreadingHTTPServer(("127.0.0.1", 0), Upstream))
        self.server.llm_enabled = True
        self.server.llm_base_url = f"http://127.0.0.1:{upstream.server_port}/v1/"
        self.server.llm_api_key = "secret-test-key"
        self.server.llm_model = "private-model"
        with urllib.request.urlopen(self.url + "/api/config") as response:
            self.assertEqual(json.load(response), {"enabled": True})
        with self.post({"model": "untrusted-model", "messages": [], "stream": True}) as response:
            self.assertEqual(json.load(response)["choices"][0]["message"]["content"], "hello")
        self.assertEqual(received["path"], "/v1/chat/completions")
        self.assertEqual(received["auth"], "Bearer secret-test-key")
        self.assertEqual(received["body"]["model"], "private-model")
        self.assertIs(received["body"]["stream"], False)

    def test_proxy_rejects_cross_origin_and_invalid_requests(self):
        self.server.llm_enabled = True
        with self.assertRaises(urllib.error.HTTPError) as error:
            self.post({"messages": []}, Origin="https://untrusted.example")
        self.assertEqual(error.exception.code, 403)
        with self.assertRaises(urllib.error.HTTPError) as error:
            self.post({"messages": "invalid"})
        self.assertEqual(error.exception.code, 400)
        with self.assertRaises(urllib.error.HTTPError) as error:
            urllib.request.urlopen(self.url + "/api/missing")
        self.assertEqual(error.exception.code, 404)


if __name__ == "__main__":
    unittest.main()
