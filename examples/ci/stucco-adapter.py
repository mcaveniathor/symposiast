#!/usr/bin/env python3
"""Return the fixed CI recipe with coverage for this job's collected evidence."""

import json
from pathlib import Path
import sys


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def analysis_for(job):
    if job["schema_version"] != 2 or job["operation"] != "analyze":
        raise ValueError("the CI fixture requires a version 2 analysis job")
    expected = json.loads((ROOT / "plugins/stucco/bundle/bundle.json").read_text())
    for field in ("name", "version", "checksum"):
        if job["release"][field] != expected["release"][field]:
            raise ValueError("the CI fixture requires the published Stucco 0.3.0 release")
    analysis = json.loads((HERE / "stucco-analysis.json").read_text())
    sources = {entry["source"]["id"]: entry["source"] for entry in job["sources"]}
    expected_sources = {source["id"]: source for source in expected["sources"]}
    for item in analysis["findings"] + analysis["recipes"]:
        for citation in item["citations"]:
            source_id = citation["source_id"]
            for field in ("url", "sha256"):
                if sources[source_id][field] != expected_sources[source_id][field]:
                    raise ValueError("the CI fixture's cited published evidence changed")

    coverage = []
    for example in job["example_inventory"]["examples"]:
        citation = example["citation"]
        recipes = [recipe["id"] for recipe in analysis["recipes"] if any(
            cited["source_id"] == citation["source_id"]
            and cited["start_line"] <= citation["end_line"]
            and cited["end_line"] >= citation["start_line"]
            for cited in recipe["citations"]
        )]
        coverage.append({
            "example_id": example["id"],
            "disposition": "adapted" if recipes else "skipped",
            "recipe_ids": recipes,
            "reason": (
                "The fixed semantic-page recipe adapts the cited published README example."
                if recipes else
                "This CI fixture covers only the cited published semantic-page example; "
                "other documentation examples are outside its deliberately narrow scope."
            ),
        })
    if not any(entry["disposition"] == "adapted" for entry in coverage):
        raise ValueError("the CI fixture requires its cited documentation example in the inventory")
    analysis["example_coverage"] = coverage
    return analysis


def main():
    try:
        analysis = analysis_for(json.load(sys.stdin))
    except (KeyError, TypeError, ValueError, OSError) as error:
        print(f"Stucco CI fixture failed: {error}", file=sys.stderr)
        return 1
    json.dump(analysis, sys.stdout)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
