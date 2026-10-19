#!/usr/bin/env python3
"""Exercise the uploaded bundle as a recipient, without provider credentials."""

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tarfile
import tempfile


def main():
    archive_path = Path(os.environ["ARCHIVE_PATH"])
    original = Path(os.environ["BUNDLE_PATH"])
    executable = shutil.which("symposiast")
    assert executable, "symposiast must be on PATH"
    with tempfile.TemporaryDirectory(prefix="guidance-recipient-") as temporary:
        root = Path(temporary)
        with tarfile.open(archive_path, "r:gz") as archive:
            for member in archive.getmembers():
                path = PurePosixPath(member.name)
                assert member.isfile(), f"Unexpected archive member: {member.name}"
                assert not path.is_absolute() and ".." not in path.parts
                assert path.parts[0] == "stucco-0.2.1"
                assert not {"plugin", "target", ".git", ".codex"}.intersection(path.parts)
            archive.extractall(root, filter="data")

        bundle = root / "stucco-0.2.1"
        data = json.loads((bundle / "bundle.json").read_text())
        assert (bundle / "bundle.json").read_bytes() == (original / "bundle.json").read_bytes()
        assert data["release"]["name"] == "stucco"
        assert data["release"]["version"] == "0.2.1"
        assert len(data["analysis"]["recipes"]) == 1
        assert all(check["status"] == "compiled" for check in data["verification"])
        for source in data["sources"]:
            assert hashlib.sha256((bundle / source["path"]).read_bytes()).hexdigest() == source["sha256"]
        consumer = bundle / "verification" / "semantic-page"
        subprocess.run([executable, "check", str(bundle), "--workspace", str(consumer)], check=True)
        runtime_env = dict(os.environ, CARGO_TARGET_DIR=str(root / "target"))
        subprocess.run(
            ["cargo", "run", "--locked", "--manifest-path", str(consumer / "Cargo.toml")],
            env=runtime_env, check=True,
        )
        home = root / "symposium"
        subprocess.run([executable, "install", str(bundle), "--symposium-home", str(home)], check=True)
        plugins = list((home / "plugins").iterdir())
        assert len(plugins) == 1
        plugin = plugins[0]
        assert (plugin / "SYMPOSIUM.toml").read_bytes() == (plugin / "Symposium.toml").read_bytes()
        assert list((plugin / "skills").glob("*/SKILL.md"))
        assert (plugin / "bundle" / "bundle.json").is_file()
    print("Portable archive, runtime assertions, and recipient plugin installation passed.")


if __name__ == "__main__":
    main()
