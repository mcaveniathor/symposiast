"""The deterministic CI adapter must honor release identity and example coverage."""

import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
ADAPTER = ROOT / "examples/ci/stucco-adapter.py"
SPEC = importlib.util.spec_from_file_location("stucco_ci_adapter", ADAPTER)
adapter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(adapter)


class FixtureAdapterTests(unittest.TestCase):
    def job(self):
        bundle = json.loads((ROOT / "plugins/stucco/bundle/bundle.json").read_text())
        return {
            "schema_version": 2, "operation": "analyze", "release": bundle["release"],
            "sources": [{"source": source} for source in bundle["sources"]],
            "example_inventory": {"examples": [
                {"id": "current-readme-id", "citation": {
                    "source_id": "e0003", "start_line": 7, "end_line": 26}},
                {"id": "other-example-id", "citation": {
                    "source_id": "e0003", "start_line": 30, "end_line": 40}},
            ]},
        }

    def test_stdout_contract_from_isolated_directory(self):
        job = self.job()
        with tempfile.TemporaryDirectory() as work:
            result = subprocess.run([str(ADAPTER)], input=json.dumps(job), cwd=work,
                                    text=True, capture_output=True, check=True)
        analysis = json.loads(result.stdout)
        template = json.loads((ROOT / "examples/ci/stucco-analysis.json").read_text())
        self.assertEqual(analysis["recipes"], template["recipes"])
        self.assertEqual(analysis["findings"], template["findings"])
        coverage = analysis["example_coverage"]
        self.assertEqual([entry["example_id"] for entry in coverage],
                         ["current-readme-id", "other-example-id"])
        self.assertEqual(coverage[0]["disposition"], "adapted")
        self.assertEqual(coverage[0]["recipe_ids"], ["semantic-page"])
        self.assertEqual(coverage[1]["disposition"], "skipped")
        self.assertEqual(coverage[1]["recipe_ids"], [])
        self.assertTrue(all(entry["reason"].strip() for entry in coverage))

    def test_optional_repository_sources_are_not_required(self):
        job = self.job()
        job["sources"] = [entry for entry in job["sources"]
                          if entry["source"]["url"].startswith("https://docs.rs/")]
        self.assertEqual(len(adapter.analysis_for(job)["example_coverage"]), 2)

    def test_changed_release_or_cited_evidence_is_rejected(self):
        original = self.job()
        for field in ("name", "version", "checksum"):
            with self.subTest(field=field):
                job = copy.deepcopy(original)
                job["release"][field] = "different"
                with self.assertRaises(ValueError):
                    adapter.analysis_for(job)
        for field in ("url", "sha256"):
            with self.subTest(field=field):
                job = copy.deepcopy(original)
                job["sources"][0]["source"][field] = "different"
                with self.assertRaises(ValueError):
                    adapter.analysis_for(job)

    def test_missing_inventory_example_or_wrong_job_is_rejected(self):
        original = self.job()
        for key, value in (("schema_version", 1), ("operation", "review"),
                           ("example_inventory", {"examples": []})):
            with self.subTest(key=key):
                job = copy.deepcopy(original)
                job[key] = value
                with self.assertRaises(ValueError):
                    adapter.analysis_for(job)


if __name__ == "__main__":
    unittest.main()
