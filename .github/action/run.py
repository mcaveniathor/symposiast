#!/usr/bin/env python3
"""GitHub Action implementation; local `package` requires no provider credentials."""

import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib
import urllib.error
import urllib.request


class ActionError(Exception):
    pass


IDENTIFIER = re.compile(r"[a-z0-9_-]{1,100}\Z")
SEMVER = re.compile(
    r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
    r"(?:-((?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)"
    r"(?:\.(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*))*))?"
    r"(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?\Z"
)
AGENTS = {"codex", "claude-code", "custom"}
CARGO_ENV = {
    "PATH", "HOME", "USERPROFILE", "SYSTEMROOT", "TEMP", "TMP", "CARGO_HOME",
    "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY",
    "NO_PROXY", "http_proxy", "https_proxy", "all_proxy", "no_proxy",
    "SSL_CERT_FILE", "SSL_CERT_DIR", "CARGO_HTTP_CAINFO",
}


def require(condition, message):
    if not condition:
        raise ActionError(message)


def clean_string(value, label):
    require(isinstance(value, str), f"{label} must be a string")
    require(not any(ord(c) < 32 or ord(c) == 127 for c in value),
            f"{label} cannot contain control characters")
    return value


def version_string(value):
    clean_string(value, "version")
    if not value:
        return ""
    if value.startswith("v"):
        value = value[1:]
    require(bool(SEMVER.fullmatch(value)),
            "version must be an exact semantic version, optionally prefixed with v")
    return value


def integer(value, name, low, high):
    require(bool(re.fullmatch(r"[0-9]+", value)), f"{name} must be an integer")
    number = int(value)
    require(low <= number <= high, f"{name} must be between {low} and {high}")
    return number


def artifact_name(value):
    clean_string(value, "artifact-name")
    require(0 < len(value) <= 200 and not re.search(r'["<>:/\\|?*]', value)
            and value not in {".", ".."}, "artifact-name is not a valid artifact name")
    return value


def absolute_path(value):
    return Path(os.path.abspath(clean_string(str(value), "path")))


def reject_symlinks(path):
    for part in [path, *path.parents]:
        require(not part.is_symlink(), "symbolic links are not allowed in bundle paths")


def empty_output(value, work):
    path = absolute_path(value) if value else work / "bundle"
    reject_symlinks(path)
    require(not path.exists() or (path.is_dir() and not any(path.iterdir())),
            "output-directory must be new or empty")
    path.mkdir(parents=True, exist_ok=True)
    return path


def executable(value):
    clean_string(value, "executable")
    found = shutil.which(value)
    require(found is not None, "requested executable was not found or is not executable")
    return str(absolute_path(found))


def inputs():
    defaults = {
        "crate": "", "version": "", "agent": "codex", "agent-executable": "",
        "reviewer": "", "reviewer-executable": "", "analysis-file": "",
        "timeout": "600", "registry-wait-seconds": "300", "rust-toolchain": "",
        "symposiast-executable": "", "output-directory": "", "artifact-name": "",
        "upload-artifact": "true", "retention-days": "14",
    }
    data = {key: clean_string(os.environ.get(
        "SYMPOSIAST_INPUT_" + key.upper().replace("-", "_"), default), key)
        for key, default in defaults.items()}
    require(bool(IDENTIFIER.fullmatch(data["crate"])), "crate must be a lowercase crates.io name")
    data["version"] = version_string(data["version"])
    require(data["agent"] in AGENTS, "agent must be codex, claude-code, or custom")
    require(not data["reviewer"] or data["reviewer"] in AGENTS,
            "reviewer must be codex, claude-code, or custom")
    require(not data["reviewer-executable"] or data["reviewer"],
            "reviewer-executable requires reviewer")
    require(data["agent"] != "custom" or data["agent-executable"] or data["analysis-file"],
            "custom agent requires agent-executable")
    require(data["reviewer"] != "custom" or data["reviewer-executable"],
            "custom reviewer requires reviewer-executable")
    data["timeout"] = integer(data["timeout"], "timeout", 1, 86400)
    data["registry-wait-seconds"] = integer(data["registry-wait-seconds"],
                                            "registry-wait-seconds", 0, 3600)
    integer(data["retention-days"], "retention-days", 1, 90)
    require(data["upload-artifact"] in {"true", "false"}, "upload-artifact must be true or false")
    if data["artifact-name"]:
        artifact_name(data["artifact-name"])
    for key in ("agent-executable", "reviewer-executable", "symposiast-executable"):
        if data[key]:
            data[key] = executable(data[key])
    if data["analysis-file"]:
        path = absolute_path(data["analysis-file"])
        require(path.is_file(), "analysis-file must name an existing file")
        data["analysis-file"] = str(path)
    return data


def sparse_path(crate):
    if len(crate) <= 2:
        return f"{len(crate)}/{crate}"
    if len(crate) == 3:
        return f"3/{crate[0]}/{crate}"
    return f"{crate[:2]}/{crate[2:4]}/{crate}"


def wait_for_registry(crate, version, seconds, *, opener=urllib.request.urlopen,
                      clock=time.monotonic, sleep=time.sleep):
    if not version or not seconds:
        return
    deadline = clock() + seconds
    request = urllib.request.Request(
        "https://index.crates.io/" + sparse_path(crate),
        headers={"User-Agent": "symposiast-release-action", "Cache-Control": "no-cache"},
    )
    print(f"Waiting up to {seconds}s for {crate} {version} in crates.io", flush=True)
    while True:
        remaining = deadline - clock()
        require(remaining > 0, "crate version did not become available before registry wait expired")
        try:
            with opener(request, timeout=min(15, remaining)) as response:
                content = response.read(32 * 1024 * 1024 + 1)
            require(len(content) <= 32 * 1024 * 1024, "registry response exceeds size limit")
            for line in content.decode("utf-8").splitlines():
                record = json.loads(line)
                if record.get("name") == crate and record.get("vers") == version:
                    require(not record.get("yanked", False), "requested crate version is yanked")
                    return
        except urllib.error.HTTPError as error:
            require(error.code == 404 or error.code == 429 or error.code >= 500,
                    f"registry request failed with HTTP {error.code}")
        except (urllib.error.URLError, TimeoutError):
            pass
        remaining = deadline - clock()
        require(remaining > 0, "crate version did not become available before registry wait expired")
        sleep(min(10, remaining))


def cargo_environment(toolchain):
    env = {name: value for name, value in os.environ.items() if name in CARGO_ENV}
    env.update(RUSTUP_TOOLCHAIN=toolchain, CARGO_BUILD_JOBS="2", CARGO_TERM_COLOR="never",
               CARGO_INCREMENTAL="0")
    return env


def command(argv, *, env=None, cwd=None):
    result = subprocess.run([str(arg) for arg in argv], env=env, cwd=cwd, check=False,
                            stdin=subprocess.DEVNULL)
    require(result.returncode == 0, "subprocess failed; see preceding command output")


def bootstrap(action, work, toolchain):
    env = cargo_environment(toolchain)
    env["CARGO_TARGET_DIR"] = str(work / "target")
    command(["rustup", "toolchain", "install", toolchain, "--profile", "minimal", "--no-self-update"],
            env=env)
    command(["cargo", "install", "--path", action, "--locked", "--root", work / "install"],
            env=env, cwd=action)
    return executable(str(work / "install/bin/symposiast"))


def regular_file(root, relative):
    require(isinstance(relative, str) and relative and "\\" not in relative,
            "invalid bundle file path")
    clean_string(relative, "bundle file path")
    pure = PurePosixPath(relative)
    require(not pure.is_absolute() and all(p not in {"", ".", ".."} for p in relative.split("/")),
            "bundle file path must be relative and normalized")
    path = root.joinpath(*pure.parts)
    reject_symlinks(path)
    require(path.is_file() and stat.S_ISREG(path.stat().st_mode), "bundle file is missing or not regular")
    return path


def digest(data):
    return hashlib.sha256(data).hexdigest()


def portable_files(bundle, crate, version):
    """Allow only manifest evidence and verified source projects, never arbitrary files."""
    metadata_path = regular_file(bundle, "bundle.json")
    data = json.loads(metadata_path.read_bytes())
    require(data.get("schema_version") == 1, "unsupported bundle schema")
    release = data["release"]
    require(release["name"] == crate, "bundle crate does not match requested crate")
    actual = version_string(release["version"])
    require(actual and actual == release["version"], "bundle version is invalid")
    require(not version or actual == version, "bundle version does not match requested version")
    require(bool(re.fullmatch(r"[a-f0-9]{64}", release["checksum"])), "invalid crate checksum")
    analysis = data.get("analysis")
    require(isinstance(analysis, dict) and analysis.get("recipes") and analysis.get("findings"),
            "bundle must have findings and recipes")
    review = data.get("review")
    require(review is None or (review.get("approved") is True and review.get("issues") == []),
            "bundle review has unresolved issues")
    files = {"bundle.json": metadata_path.read_bytes()}
    require(data.get("sources"), "bundle must include source evidence")
    for source in data["sources"]:
        relative = source["path"]
        require(relative.startswith("sources/"), "evidence files must be inside sources/")
        contents = regular_file(bundle, relative).read_bytes()
        require(digest(contents) == source["sha256"], "source checksum mismatch")
        files[relative] = contents
    checks = data.get("verification", [])
    by_id = {record["recipe_id"]: record for record in checks}
    recipes = analysis["recipes"]
    require(len(by_id) == len(checks) == len(recipes), "every recipe needs one verification record")
    for recipe in recipes:
        recipe_id = recipe["id"]
        require(bool(IDENTIFIER.fullmatch(recipe_id)), "invalid recipe ID")
        record = by_id.get(recipe_id, {})
        require(record.get("status") == "compiled", "every recipe must pass compilation")
        directory = f"verification/{recipe_id}"
        require(record.get("log_path") == f"{directory}/check.log", "unexpected verification log path")
        for suffix in ("Cargo.toml", "Cargo.lock", "src/main.rs", "check.log"):
            relative = f"{directory}/{suffix}"
            files[relative] = regular_file(bundle, relative).read_bytes()
        require(files[f"{directory}/src/main.rs"].decode("utf-8") == recipe["code"],
                "recipe source differs from verified analysis")
        require(digest(files[f"{directory}/Cargo.lock"]) == record.get("lockfile_sha256"),
                "compiled recipe lockfile checksum is missing or changed")
        manifest = tomllib.loads(files[f"{directory}/Cargo.toml"].decode("utf-8"))
        expected_dependencies = {
            crate: {"version": f"={actual}", "default-features": recipe["default_features"],
                    "features": recipe["features"]},
        }
        expected_dependencies.update({name: {
            "version": dependency["version"], "default-features": dependency["default_features"],
            "features": dependency["features"],
        } for name, dependency in recipe["dependencies"].items()})
        expected_manifest = {
            "package": {"name": f"recipe-{recipe_id}", "version": "0.0.0",
                        "edition": "2024", "publish": False},
            "workspace": {}, "dependencies": expected_dependencies,
        }
        require(manifest == expected_manifest, "recipe manifest differs from verified analysis")
    return actual, files


def package(bundle, crate, version, archive, cli):
    bundle = absolute_path(bundle)
    reject_symlinks(bundle)
    actual, files = portable_files(bundle, crate, version)
    # The CLI additionally validates citation ranges and Rust's canonical recipe fingerprints.
    command([cli, "check", bundle, "--workspace", bundle])
    archive = absolute_path(archive)
    reject_symlinks(archive)
    require(not archive.exists(), "archive path already exists")
    archive.parent.mkdir(parents=True, exist_ok=True)
    # No executable modes, user IDs, absolute paths, symlinks, target caches, or runtime manifests.
    with archive.open("xb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", filename="", mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode="w") as tar:
                for relative, contents in sorted(files.items()):
                    info = tarfile.TarInfo(f"{crate}-{actual}/{relative}")
                    info.size = len(contents)
                    info.mode = 0o644
                    tar.addfile(info, io.BytesIO(contents))
    return actual


def write_outputs(values):
    for value in values.values():
        clean_string(str(value), "output")
    destination = os.environ.get("GITHUB_OUTPUT")
    if destination:
        with open(destination, "a", encoding="utf-8") as output:
            output.writelines(f"{key}={value}\n" for key, value in values.items())
    print(f"Packaged {values['artifact-name']}", flush=True)


def run():
    require(sys.platform == "linux" and os.environ.get("RUNNER_OS") == "Linux",
            "this action requires a Linux runner with Python 3.11 or later")
    data = inputs()
    action = absolute_path(os.environ.get("SYMPOSIAST_ACTION_PATH", Path(__file__).parents[2]))
    toolchain = data["rust-toolchain"] or tomllib.loads(
        (action / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    require(bool(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,99}", toolchain)),
            "rust-toolchain must be a rustup toolchain name")
    temporary = os.environ.get("RUNNER_TEMP")
    require(temporary and Path(temporary).is_dir(), "RUNNER_TEMP must name an existing directory")
    work = Path(tempfile.mkdtemp(prefix="symposiast-action-", dir=temporary))
    bundle = empty_output(data["output-directory"], work)
    wait_for_registry(data["crate"], data["version"], data["registry-wait-seconds"])
    cli = data["symposiast-executable"] or bootstrap(action, work, toolchain)
    argv = [cli, "build", data["crate"], "--output", str(bundle), "--agent", data["agent"],
            "--timeout", str(data["timeout"])]
    for key, flag in (("version", "--version"), ("agent-executable", "--agent-executable"),
                      ("reviewer", "--reviewer"), ("reviewer-executable", "--reviewer-executable"),
                      ("analysis-file", "--analysis")):
        if data[key]:
            argv.extend([flag, data[key]])
    env = os.environ.copy()
    env["RUSTUP_TOOLCHAIN"] = toolchain
    command(argv, env=env)
    actual, _ = portable_files(bundle, data["crate"], data["version"])
    name = artifact_name(data["artifact-name"] or f"symposiast-{data['crate']}-{actual}")
    archive = work / f"{data['crate']}-{actual}.tar.gz"
    package(bundle, data["crate"], actual, archive, cli)
    if not data["symposiast-executable"] and os.environ.get("GITHUB_PATH"):
        with open(os.environ["GITHUB_PATH"], "a", encoding="utf-8") as paths:
            paths.write(f"{Path(cli).parent}\n")
    write_outputs({"bundle-path": bundle, "archive-path": archive,
                   "crate-version": actual, "artifact-name": name})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("run", help="Read SYMPOSIAST_INPUT_* environment variables and build once")
    packing = subparsers.add_parser("package", help="Validate and package an existing compiled bundle")
    packing.add_argument("bundle", type=Path)
    packing.add_argument("--crate", required=True)
    packing.add_argument("--version", default="")
    packing.add_argument("--archive", type=Path, required=True)
    packing.add_argument("--symposiast-executable", required=True)
    args = parser.parse_args()
    try:
        if args.command == "run":
            run()
        else:
            require(bool(IDENTIFIER.fullmatch(args.crate)), "invalid crate name")
            actual = package(args.bundle, args.crate, version_string(args.version), args.archive,
                             executable(args.symposiast_executable))
            print(f"Packaged {args.crate} {actual}")
    except (ActionError, OSError, ValueError, KeyError, TypeError) as error:
        # Input and subprocess diagnostics never include environment values or command lines.
        print(f"Symposiast action failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
