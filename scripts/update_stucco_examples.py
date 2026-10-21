#!/usr/bin/env python3
"""Refresh Stucco downloads and the examples guide from the checked-in bundle.

Use --check in CI to reject stale recipe code, manifests, lockfiles, or prose.
The site/downloads/stucco directory and Stucco section of the guide are generated.
"""

import argparse
from collections import Counter
import json
from pathlib import Path
import shutil


ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "plugins/stucco/bundle"
DOWNLOADS = ROOT / "site/downloads/stucco"
GUIDE = ROOT / "site/content/examples.md"


def generated():
    bundle = json.loads((BUNDLE / "bundle.json").read_text())
    version = bundle["release"]["version"]
    sources = {source["id"]: source for source in bundle["sources"]}
    analysis = bundle["analysis"]
    coverage = Counter(entry["disposition"] for entry in analysis["example_coverage"])
    files = {}
    section = f'''## Stucco {version}

This documentation site is built with Stucco {version}. The six recipes below cover static asset export, Axum fragments, derived tables, validation feedback, server-selected themes, and native status/detail components.

### Install the Stucco plugin

With Symposiast and Symposium configured, run from this checkout:

```sh
symposiast install plugins/stucco/bundle
cd /path/to/consumer
cargo agents sync
```

Guidance and MCP retrieval match Stucco **{version}** exactly. The version hook warns about other releases. Restart the consuming MCP session after installation and inspect the [plugin guide](https://github.com/mcaveniathor/symposiast/blob/main/plugins/stucco/README.md).

The bundle contains {len(sources)} frozen sources, including both upstream license texts, at release commit `{bundle['release']['repository_commit']}`. Its {len(bundle['example_inventory']['examples'])} inventoried documentation examples have {coverage['adapted']} adapted, {coverage['supporting']} supporting, and {coverage['skipped']} skipped decisions with reasons. The generated plugin exposes that complete coverage report. This is coverage of the collected evidence, not the entire crate.

Every recipe has a recorded compilation check and a pinned consumer lockfile. Local executable assertions are recorded separately in the [validation notes](https://github.com/mcaveniathor/symposiast/blob/main/docs/validation.md). They do not establish browser behavior or application storage integration.

'''
    for recipe in analysis["recipes"]:
        name = recipe["id"]
        for filename in ["Cargo.toml", "Cargo.lock", "src/main.rs"]:
            files[f"{name}/{filename}"] = (BUNDLE / "verification" / name / filename).read_bytes()
        if files[f"{name}/src/main.rs"].decode() != recipe["code"]:
            raise ValueError(f"{name}: consumer code differs from its analysis; verify the bundle first")
        section += f"### {recipe['title']}\n\n{recipe['description']}\n\n"
        section += " · ".join(f"[{filename}](examples/stucco/{name}/{filename})"
                              for filename in ["Cargo.toml", "Cargo.lock", "src/main.rs"]) + "\n\n"
        section += "```toml\n" + files[f"{name}/Cargo.toml"].decode().rstrip() + "\n```\n\n"
        section += "```rust\n" + recipe["code"].rstrip() + "\n```\n\n"
        section += f"```sh\ncargo run --locked --manifest-path site/downloads/stucco/{name}/Cargo.toml"
        if name == "static-page":
            section += " -- /tmp/stucco-preview"
        section += "\n```\n\n"
        if name == "axum-fragments":
            section += "The default run completes after in-memory HTTP checks. Append `-- --serve` to start the demo server at `http://127.0.0.1:3000`.\n\n"
        citations = []
        for citation in recipe["citations"]:
            source = sources[citation["source_id"]]
            start, end = citation["start_line"], citation["end_line"]
            url = source["url"]
            if url.startswith("https://github.com/"):
                url += f"#L{start}-L{end}"
            citations.append(f"[{source['kind']}, lines {start}–{end}]({url})")
        section += "Release evidence: " + " · ".join(citations) + ".\n\n"
    before, separator, _ = GUIDE.read_text().partition("## Stucco ")
    if not separator:
        raise ValueError("the examples guide has no Stucco section")
    return files, before + section.rstrip() + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    files, guide = generated()
    if args.check:
        actual = {path.relative_to(DOWNLOADS).as_posix(): path.read_bytes()
                  for path in DOWNLOADS.rglob("*") if path.is_file()}
        if actual != files or GUIDE.read_text() != guide:
            parser.exit(1, "Stucco examples are stale; run python3 scripts/update_stucco_examples.py\n")
        print("Stucco downloads and guide match the verified bundle.")
    else:
        if DOWNLOADS.exists():
            shutil.rmtree(DOWNLOADS)
        for relative, content in files.items():
            path = DOWNLOADS / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        GUIDE.write_text(guide)
        print(f"Updated {len(files)} downloadable files and the Stucco examples guide.")


if __name__ == "__main__":
    main()
