#!/usr/bin/env python3
"""Exercise a relocated Stucco export through the real Symposium CLI.

Uses an isolated SYMPOSIUM_HOME and temporary consumer workspaces. No agent
authentication or generation is needed. Install Symposium 0.5.1 first.
"""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(args, cwd, env):
    result = subprocess.run(args, cwd=cwd, env=env, text=True,
                            capture_output=True, timeout=120)
    if result.returncode:
        raise RuntimeError(f"{args[0]} failed:\n{result.stdout}\n{result.stderr}")
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--symposiast", default="target/debug/symposiast")
    parser.add_argument("--cargo-agents", default="cargo-agents")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    symposiast = str(Path(args.symposiast).resolve())
    symposium = shutil.which(args.cargo_agents)
    if not symposium:
        parser.error("cargo-agents not found; install symposium 0.5.1 or pass --cargo-agents")
    symposium = str(Path(symposium).resolve())

    with tempfile.TemporaryDirectory(prefix="symposiast-registry-check-") as scratch:
        root = Path(scratch)
        env = {**os.environ, "SYMPOSIUM_HOME": str(root / "home"),
               "CARGO_NET_OFFLINE": "true"}
        (root / "home/logs").mkdir(parents=True)
        # Copy only portable bundle inputs, never caches or local runtime renders.
        source = root / "bundle"
        shutil.copytree(repo / "plugins/stucco/bundle", source,
                        ignore=shutil.ignore_patterns("target", "plugin"))
        run([symposiast, "export-registry", str(source), "--output",
             str(root / "export/stucco-0-2-1")], repo, env)
        registry = root / "relocated-registry"
        (root / "export").rename(registry)
        # Use the real pinned consumer project for the matching-version check.
        consumer = root / "consumer"
        shutil.copytree(source / "verification/static-page", consumer)
        shutil.rmtree(source)
        (root / "home/config.toml").write_text(f'''auto-update = "off"
auto-sync = false
agents-syncing = false
hook-scope = "project"
[defaults]
symposium-recommendations = false
user-plugins = false
[[agent]]
name = "claude"
[[agent]]
name = "codex"
[[registry]]
name = "pilot"
path = {json.dumps(str(registry))}
''')
        validation = json.loads(run([symposium, "agents", "plugin", "validate",
                                     str(registry), "--no-check-crates", "--json"], consumer, env))
        assert len(validation) == 2 and all(item.get("valid") for item in validation), validation
        entry = registry / "stucco-0-2-1"
        skill = next((entry / "skills").iterdir())
        for _ in range(2):
            run([symposium, "agents", "sync", "--json"], consumer, env)
            for agent_dir in [".claude", ".agents"]:
                installed = consumer / agent_dir / "skills" / skill.name
                for file in skill.rglob("*"):
                    if file.is_file():
                        assert (installed / file.relative_to(skill)).read_bytes() == file.read_bytes()
        # Exercise a version change through Cargo metadata and Symposium sync.
        # This tiny path crate is a version-matching fixture, not a Stucco build.
        mismatch = root / "mismatch"
        (mismatch / "src").mkdir(parents=True)
        (mismatch / "Cargo.toml").write_text('[package]\nname = "stucco"\nversion = "0.2.2"\nedition = "2024"\n')
        (mismatch / "src/lib.rs").write_text("")
        (consumer / "Cargo.toml").write_text('''[package]
name = "registry-consumer"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
stucco = { path = "../mismatch" }
''')
        (consumer / "Cargo.lock").unlink()
        run([symposium, "agents", "sync", "--json"], consumer, env)
        for agent_dir in [".claude", ".agents"]:
            assert not (consumer / agent_dir / "skills" / skill.name).exists()
        print("Registry validation, relocation, Claude/Codex skill installation, repeated sync, and version-mismatch removal passed.")


if __name__ == "__main__":
    main()
