#!/usr/bin/env python3
"""Build generated axum consumers and exercise their actual localhost servers.

Run from any directory. Uses CARGO_TARGET_DIR when set; otherwise target/docs-runtime.
Port 3000 must be free. Only servers started by this process are stopped.
"""

import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
USER_ID = "00000000-0000-0000-0000-000000000001"

# recipe, method, path, request body, headers, expected status, expected response
CASES = {
    "r01": [("GET", "/", None, {}, 200, "Hello, World!"),
            ("POST", "/users", b'{"username":"Ada"}', {"Content-Type": "application/json"}, 201, {"id": 1337, "username": "Ada"})],
    "r02": [("GET", "/foo/a/b", None, {}, 200, "a/b"),
            ("GET", "/foo/bar", None, {}, 200, "static route"),
            ("GET", "/api/missing", None, {}, 404, "API fallback"),
            ("GET", "/shared/missing", None, {}, 404, "outer fallback"),
            ("PUT", "/methods", b"", {}, 405, "PUT is not allowed for /methods")],
    "r03": [("GET", "/", None, {}, 200, "Hello from shared state")],
    "r04": [("GET", f"/users/{USER_ID}/things?page=2&per_page=5", None, {}, 200,
             {"user_id": USER_ID, "page": 2, "per_page": 5}),
            ("GET", "/users/invalid", None, {}, 400, None)],
    "r05": [("POST", "/echo", b'{"username":"Ada"}', {"Content-Type": "application/json"}, 200, {"username": "Ada"}),
            ("POST", "/echo", b"{", {"Content-Type": "application/json"}, 400, None)],
    "r06": [("POST", "/echo", b"hello", {"User-Agent": "example-check"}, 200, "hello"),
            ("POST", "/echo", b"", {"User-Agent": "example-check"}, 400, "Body must not be empty")],
    "r07": [("GET", "/profile", None, {}, 401, ""),
            ("GET", "/profile", None, {"Authorization": "Bearer demo-token"}, 200, "Hello, demo")],
    "r08": [("GET", "/service", None, {}, 500, "Service failed"),
            ("GET", "/slow", None, {}, 408, "GET /slow could not be completed")],
    "r09": [("GET", "/", None, {}, 200, "Hello"),
            ("GET", "/raw", None, {}, 404, "not found"),
            ("GET", "/choice?format=html", None, {}, 200, "<p>Hello</p>"),
            ("POST", "/create", b"", {}, 400, ""),
            ("POST", "/create", b"", {"x-name": "Ada"}, 201, "")],
    "r10": [("GET", "/form?value=hello", None, {}, 200, "value=hello"),
            ("POST", "/form", b"value=hello", {"Content-Type": "application/x-www-form-urlencoded"}, 200, "value=hello")],
    "r12": [("GET", "/legacy?keep=yes", None, {}, 200, "Current endpoint")],
    "r13": [("GET", "/", None, {}, 200, "Application: example")],
}


def request(method, path, body, headers):
    try:
        response = urlopen(Request("http://127.0.0.1:3000" + path, data=body, headers=headers, method=method), timeout=5)
    except HTTPError as error:
        response = error
    with response:
        return response.status, response.headers, response.read().decode()


def main():
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target/docs-runtime")).resolve()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2")
    recipes = json.loads((ROOT / "plugins/axum/bundle/bundle.json").read_text())["analysis"]["recipes"]
    if {recipe["id"] for recipe in recipes} != set(CASES) | {"r11"}:
        raise RuntimeError("Generated recipe set changed; review and update the HTTP scenarios")
    for recipe in recipes:
        rid = recipe["id"]
        manifest = ROOT / "site/downloads/axum" / rid / "Cargo.toml"
        subprocess.run(["cargo", "build", "--locked", "--manifest-path", str(manifest)], env=env, check=True)
        executable = target / "debug" / f"recipe-{rid}"
        if rid == "r11":
            subprocess.run([str(executable)], check=True, timeout=10)
            print("r11: body-limit assertions passed", flush=True)
            continue
        # Refuse to send probes to an unrelated process occupying the demo port.
        with socket.socket() as probe:
            probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            probe.bind(("127.0.0.1", 3000))
        with tempfile.TemporaryFile(mode="w+") as log:
            process = subprocess.Popen([str(executable)], stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 10
                while True:
                    if process.poll() is not None:
                        log.seek(0)
                        raise RuntimeError(f"{rid} exited before serving: {log.read()}")
                    try:
                        request("GET", "/", None, {})
                        break
                    except (URLError, ConnectionError):
                        if time.monotonic() >= deadline:
                            raise TimeoutError(f"{rid} did not start")
                        time.sleep(0.05)
                for method, path, body, headers, status, expected in CASES[rid]:
                    actual_status, response_headers, actual = request(method, path, body, headers)
                    assert actual_status == status, (rid, path, actual_status, status, actual)
                    if isinstance(expected, dict):
                        assert response_headers.get_content_type() == "application/json"
                        actual = json.loads(actual)
                    if expected is not None:
                        assert actual == expected, (rid, path, actual, expected)
                print(f"{rid}: {len(CASES[rid])} HTTP assertions passed", flush=True)
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


if __name__ == "__main__":
    main()
