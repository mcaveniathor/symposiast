"""Offline contract tests for inputs, subprocess boundaries, and portable artifacts."""

import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SPEC = importlib.util.spec_from_file_location("symposiast_action", HERE / "run.py")
action = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(action)
FIXTURE = ROOT / "plugins/stucco/bundle"


class ActionTests(unittest.TestCase):
    def test_ci_analysis_uses_only_published_release_evidence(self):
        analysis = json.loads((ROOT / "examples/ci/stucco-analysis.json").read_text())
        bundle = json.loads((FIXTURE / "bundle.json").read_text())
        published = {
            source["id"]: source for source in bundle["sources"]
            if source["url"].startswith("https://docs.rs/crate/stucco/0.3.0/source/")
        }
        for item in analysis["findings"] + analysis["recipes"]:
            self.assertTrue(item["citations"])
            for citation in item["citations"]:
                # Removing all optional repository evidence must leave every citation valid.
                self.assertIn(citation["source_id"], published)
                source = published[citation["source_id"]]
                lines = (FIXTURE / source["path"]).read_text().splitlines()
                self.assertGreaterEqual(citation["start_line"], 1)
                self.assertLessEqual(citation["start_line"], citation["end_line"])
                self.assertLessEqual(citation["end_line"], len(lines))

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="symposiast-action-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.ledger = self.root / "calls.jsonl"
        self.outputs = self.root / "outputs"
        self.cli = self.root / "symposiast fixture"
        self.cli.write_text(
            "#!/usr/bin/env python3\n"
            "import json, os, pathlib, shutil, sys\n"
            f"ledger = pathlib.Path({str(self.ledger)!r})\n"
            "with ledger.open('a') as log:\n"
            "    log.write(json.dumps({'args': sys.argv[1:], 'toolchain': "
            "os.environ.get('RUSTUP_TOOLCHAIN')}) + '\\n')\n"
            "if sys.argv[1] == 'build':\n"
            "    if os.environ.get('FIXTURE_FAIL'):\n"
            "        sys.exit(9)\n"
            "    destination = pathlib.Path(sys.argv[sys.argv.index('--output') + 1])\n"
            f"    shutil.copytree({str(FIXTURE)!r}, destination, dirs_exist_ok=True)\n"
            "    if os.environ.get('FIXTURE_BAD_STATUS'):\n"
            "        path = destination / 'bundle.json'\n"
            "        data = json.loads(path.read_text())\n"
            "        data['verification'][0]['status'] = 'failed'\n"
            "        path.write_text(json.dumps(data))\n"
            "elif sys.argv[1] == 'check' and os.environ.get('FIXTURE_CHECK_FAIL'):\n"
            "    sys.exit(4)\n"
        )
        self.cli.chmod(0o755)
        self.env = {name: value for name, value in os.environ.items()
                    if not name.startswith("SYMPOSIAST_INPUT_")}
        self.env.update({
            "RUNNER_OS": "Linux", "RUNNER_ENVIRONMENT": "github-hosted",
            "RUNNER_TEMP": str(self.root), "GITHUB_OUTPUT": str(self.outputs),
            "SYMPOSIAST_ACTION_PATH": str(ROOT), "SYMPOSIAST_INPUT_CRATE": "stucco",
            "SYMPOSIAST_INPUT_VERSION": "v0.3.0", "SYMPOSIAST_INPUT_REGISTRY_WAIT_SECONDS": "0",
            "SYMPOSIAST_INPUT_SYMPOSIAST_EXECUTABLE": str(self.cli),
            "SYMPOSIAST_INPUT_UPLOAD_ARTIFACT": "false",
        })

    def run_action(self, **inputs):
        env = self.env.copy()
        env.update({"SYMPOSIAST_INPUT_" + key.upper().replace("-", "_"): str(value)
                    for key, value in inputs.items()})
        return subprocess.run([sys.executable, str(HERE / "run.py"), "run"], env=env,
                              cwd=self.root, text=True, capture_output=True)

    def calls(self):
        if not self.ledger.exists():
            return []
        return [json.loads(line) for line in self.ledger.read_text().splitlines()]

    def output_values(self):
        return dict(line.split("=", 1) for line in self.outputs.read_text().splitlines())

    def copy_bundle(self):
        target = self.root / "bundle"
        shutil.copytree(FIXTURE, target)
        return target

    def assert_failed_before_build(self, result):
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertFalse(self.outputs.exists())
        self.assertEqual(self.calls(), [])

    def test_success_builds_once_and_packages_complete_allowlist(self):
        result = self.run_action()
        self.assertEqual(result.returncode, 0, result.stderr)
        values = self.output_values()
        self.assertEqual(values["crate-version"], "0.3.0")
        self.assertEqual(values["artifact-name"], "symposiast-stucco-0.3.0")
        self.assertEqual([call["args"][0] for call in self.calls()], ["build", "check"])
        self.assertEqual(self.calls()[0]["toolchain"], "1.99.1")
        args = self.calls()[0]["args"]
        self.assertEqual(args[args.index("--version") + 1], "0.3.0")
        with tarfile.open(values["archive-path"]) as archive:
            metadata = json.load(archive.extractfile("stucco-0.3.0/bundle.json"))
            expected = {"stucco-0.3.0/bundle.json"}
            expected.update("stucco-0.3.0/" + source["path"] for source in metadata["sources"])
            for recipe in metadata["analysis"]["recipes"]:
                expected.update(f"stucco-0.3.0/verification/{recipe['id']}/{suffix}"
                                for suffix in ("Cargo.toml", "Cargo.lock", "src/main.rs", "check.log"))
            self.assertEqual(set(archive.getnames()), expected)
            for member in archive.getmembers():
                self.assertTrue(member.isfile())
                self.assertEqual(member.mode, 0o644)
                self.assertEqual(member.uid, 0)
                relative = member.name.removeprefix("stucco-0.3.0/")
                self.assertEqual(archive.extractfile(member).read(), (FIXTURE / relative).read_bytes())

    def test_inputs_are_argv_data_not_shell_code(self):
        agent = self.root / "agent $(touch OWNED); weird name"
        agent.write_text("#!/bin/sh\nexit 0\n")
        agent.chmod(0o755)
        analysis = self.root / "analysis $(touch OWNED) 'file.json"
        analysis.write_text("{}")
        directory = self.root / "output $(touch OWNED); bundle"
        result = self.run_action(**{
            "agent": "custom", "agent-executable": agent, "analysis-file": analysis,
            "output-directory": directory, "artifact-name": "literal $(touch OWNED)",
        })
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse((self.root / "OWNED").exists())
        args = self.calls()[0]["args"]
        self.assertEqual(args[args.index("--agent-executable") + 1], str(agent))
        self.assertEqual(args[args.index("--analysis") + 1], str(analysis))
        self.assertEqual(args[args.index("--output") + 1], str(directory))

    def test_invalid_inputs_never_invoke_build(self):
        cases = [
            {"crate": "clap;touch OWNED"}, {"crate": "../clap"}, {"crate": "Clap"},
            {"version": "v"}, {"version": "clap-v1.2.3"}, {"version": "^1.2.3"}, {"version": "1.2.3-01"},
            {"version": "1.2.3\nowned=yes"}, {"timeout": "0"}, {"timeout": "NaN"},
            {"registry-wait-seconds": "3601"}, {"retention-days": "91"},
            {"upload-artifact": "TRUE"}, {"artifact-name": "../../secret"},
            {"artifact-name": "safe\nother=bad"}, {"agent": "unknown"},
            {"agent": "custom"}, {"reviewer": "custom"}, {"reviewer-executable": str(self.cli)},
            {"rust-toolchain": "stable --command evil"}, {"analysis-file": "missing.json"},
        ]
        for values in cases:
            with self.subTest(values=values):
                self.assert_failed_before_build(self.run_action(**values))

    def test_rejects_nonempty_and_symlink_output(self):
        target = self.root / "existing"
        target.mkdir()
        (target / "precious").write_text("keep")
        self.assert_failed_before_build(self.run_action(**{"output-directory": target}))
        self.assertEqual((target / "precious").read_text(), "keep")
        empty = self.root / "empty"
        empty.mkdir()
        link = self.root / "linked"
        link.symlink_to(empty, target_is_directory=True)
        self.assert_failed_before_build(self.run_action(**{"output-directory": link}))
        self.assert_failed_before_build(self.run_action(**{"output-directory": link / "child"}))

    def test_unsupported_runner_fails_before_build(self):
        self.env["RUNNER_OS"] = "Windows"
        self.assert_failed_before_build(self.run_action())

    def test_build_failure_is_not_retried_and_does_not_publish_outputs(self):
        self.env["FIXTURE_FAIL"] = "true"
        result = self.run_action()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([call["args"][0] for call in self.calls()], ["build"])
        self.assertFalse(self.outputs.exists())
        self.assertEqual(list(self.root.rglob("*.tar.gz")), [])

    def test_noncompiled_recipe_blocks_archive_and_outputs(self):
        self.env["FIXTURE_BAD_STATUS"] = "true"
        result = self.run_action()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.outputs.exists())
        self.assertEqual(list(self.root.rglob("*.tar.gz")), [])

    def test_cli_integrity_failure_blocks_archive_and_outputs(self):
        self.env["FIXTURE_CHECK_FAIL"] = "true"
        result = self.run_action()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([call["args"][0] for call in self.calls()], ["build", "check"])
        self.assertFalse(self.outputs.exists())
        self.assertEqual(list(self.root.rglob("*.tar.gz")), [])

    def test_package_excludes_runtime_files_caches_credentials_and_executables(self):
        bundle = self.copy_bundle()
        for relative in ("plugin/SYMPOSIUM.toml", "verification/target/debug/recipe",
                         ".env", "credentials.json", "sources/uncited.txt"):
            path = bundle / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("THIS_TEST_SECRET_MUST_NEVER_BE_PACKAGED")
            path.chmod(0o755)
        archive = self.root / "portable.tar.gz"
        action.package(bundle, "stucco", "0.3.0", archive, str(self.cli))
        with tarfile.open(archive) as tar:
            self.assertFalse(any("THIS_TEST_SECRET_MUST_NEVER_BE_PACKAGED" in tar.extractfile(item).read().decode()
                                 for item in tar.getmembers()))
            self.assertFalse(any("target/" in name or "plugin/" in name for name in tar.getnames()))

    def test_rejects_changed_sources_code_lockfiles_and_manifests(self):
        bundle = self.copy_bundle()
        files = ("sources/e0001.txt", "verification/static-page/src/main.rs",
                 "verification/static-page/Cargo.lock", "verification/static-page/Cargo.toml")
        for relative in files:
            with self.subTest(path=relative):
                path = bundle / relative
                original = path.read_bytes()
                if relative.endswith("Cargo.toml"):
                    path.write_bytes(original.replace(b'version = "=0.3.0"', b'version = "*"'))
                else:
                    path.write_bytes(original + b"\n# changed\n")
                try:
                    with self.assertRaises((action.ActionError, ValueError)):
                        action.portable_files(bundle, "stucco", "0.3.0")
                finally:
                    path.write_bytes(original)

    def test_rejects_metadata_mismatch_missing_checks_and_review_issues(self):
        bundle = self.copy_bundle()
        path = bundle / "bundle.json"
        original = json.loads(path.read_text())
        changes = [
            lambda data: data["release"].update(version="0.2.0"),
            lambda data: data["release"].update(name="different"),
            lambda data: data.update(verification=[]),
            lambda data: data["verification"][0].update(lockfile_sha256=None),
            lambda data: data["verification"][0].update(log_path=".env"),
            lambda data: data.update(review={"approved": False, "issues": ["fix"]}),
            lambda data: data.update(analysis=None),
            lambda data: data["sources"][0].update(path="../secret"),
        ]
        for change in changes:
            data = copy.deepcopy(original)
            change(data)
            path.write_text(json.dumps(data))
            with self.assertRaises(action.ActionError):
                action.portable_files(bundle, "stucco", "0.3.0")

    def test_rejects_symlink_evidence(self):
        bundle = self.copy_bundle()
        path = bundle / "sources/e0001.txt"
        path.unlink()
        path.symlink_to(FIXTURE / "sources/e0001.txt")
        with self.assertRaises(action.ActionError):
            action.portable_files(bundle, "stucco", "0.3.0")

    def test_bootstrap_environment_excludes_credentials_and_rustflags(self):
        with patch.dict(os.environ, {"OPENAI_API_KEY": "private", "ANTHROPIC_API_KEY": "private",
                                     "GH_TOKEN": "private", "RUSTFLAGS": "-C link-arg=bad",
                                     "HTTPS_PROXY": "http://proxy.example"}):
            env = action.cargo_environment("1.99.1")
        for name in ("OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GH_TOKEN", "RUSTFLAGS"):
            self.assertNotIn(name, env)
        self.assertEqual(env["HTTPS_PROXY"], "http://proxy.example")
        self.assertEqual(env["RUSTUP_TOOLCHAIN"], "1.99.1")


class RegistryTests(unittest.TestCase):
    def test_exact_version_wait_retries_only_registry(self):
        requests = []
        now = [0]

        def opener(request, timeout):
            requests.append((request.full_url, timeout))
            if len(requests) == 1:
                raise urllib.error.HTTPError(request.full_url, 404, "not ready", {}, None)
            return io.BytesIO(b'{"name":"stucco","vers":"0.3.0","yanked":false}\n')

        def sleep(seconds):
            now[0] += seconds

        action.wait_for_registry("stucco", "0.3.0", 30, opener=opener,
                                 clock=lambda: now[0], sleep=sleep)
        self.assertEqual(len(requests), 2)
        self.assertEqual(requests[0][0], "https://index.crates.io/st/uc/stucco")

    def test_timeout_is_bounded_and_latest_or_disabled_never_polls(self):
        now = [0]

        def sleep(seconds):
            now[0] += seconds

        with self.assertRaises(action.ActionError):
            action.wait_for_registry("stucco", "0.3.0", 12, opener=lambda *a, **kw: io.BytesIO(b""),
                                     clock=lambda: now[0], sleep=sleep)
        self.assertEqual(now[0], 12)
        with patch.object(action.urllib.request, "urlopen") as opener:
            action.wait_for_registry("stucco", "", 300, opener=opener)
            action.wait_for_registry("stucco", "0.3.0", 0, opener=opener)
            opener.assert_not_called()

    def test_yanked_version_and_forbidden_response_fail_without_retries(self):
        responses = [io.BytesIO(b'{"name":"stucco","vers":"0.3.0","yanked":true}\n'),
                     urllib.error.HTTPError("url", 403, "forbidden", {}, None)]
        for response in responses:
            with patch.object(action.urllib.request, "urlopen", side_effect=(
                response if isinstance(response, Exception) else None)) as opener:
                opener.return_value = response
                with self.assertRaises(action.ActionError):
                    action.wait_for_registry("stucco", "0.3.0", 30, opener=opener)
                self.assertEqual(opener.call_count, 1)


if __name__ == "__main__":
    unittest.main()
