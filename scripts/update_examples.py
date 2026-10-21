#!/usr/bin/env python3
"""Generate clap, axum, and Bevy downloads and prose from verified bundles.

Run after Symposiast analysis/verification. --check rejects stale published files.
Stucco continues to use update_stucco_examples.py for its custom run instructions.
"""

import argparse
from collections import Counter
import hashlib
import html
import json
from pathlib import Path
import re
import shutil

from update_stucco_examples import generated as stucco_generated

ROOT = Path(__file__).resolve().parents[1]
GUIDE = ROOT / "site/content/examples.md"
CRATES = ("clap", "axum", "bevy")
LABELS = {"clap": "clap", "axum": "axum", "bevy": "Bevy"}


def prose(value):
    """Analysis strings are plain text, including Rust generics and macro syntax."""
    return re.sub(r"([\\`*_{}\[\]])", r"\\\1", html.escape(value, quote=False))


def evidence(citations, sources):
    links = []
    for citation in citations:
        source = sources[citation["source_id"]]
        start, end = citation["start_line"], citation["end_line"]
        url = source["url"]
        if url.startswith("https://github.com/"):
            url += f"#L{start}-L{end}"
        links.append(f"[{source['kind']}, lines {start}–{end}]({url})")
    return " · ".join(links)


def generated_crate(name):
    root = ROOT / "plugins" / name / "bundle"
    bundle = json.loads((root / "bundle.json").read_text())
    analysis = bundle["analysis"]
    version = bundle["release"]["version"]
    sources = {source["id"]: source for source in bundle["sources"]}
    checks = {check["recipe_id"]: check for check in bundle["verification"]}
    coverage = Counter(entry["disposition"] for entry in analysis["example_coverage"])
    files = {"coverage.json": (json.dumps({
        "release": {"name": name, "version": version},
        "inventory": bundle["example_inventory"],
        "coverage": analysis["example_coverage"],
    }, indent=2) + "\n").encode()}
    section = f"## {LABELS[name]} {version}\n\n"
    section += (
        f"Symposiast generated this guidance through its Codex adapter from frozen release evidence. "
        f"The bundle contains {len(sources)} sources and {len(analysis['recipes'])} recipes. "
        f"Its {len(bundle['example_inventory']['examples'])} inventoried documentation examples have "
        f"{coverage['adapted']} adapted, {coverage['supporting']} supporting, and {coverage['skipped']} skipped decisions. "
        f"Inspect the [complete inventory and coverage reasons](examples/{name}/coverage.json). "
        "Standalone upstream example programs are also evidence, but are not counted as documentation code blocks. "
        "All recipes passed compilation after analysis; the agent's original pre-verification summary is preserved in the bundle.\n\n"
    )
    if name == "clap":
        section += "Clap uses external includes in its rustdoc. The inventory preserves literal fragments around these boundaries; skipped fragments do not establish coverage of the included examples.\n\n"
    if name == "bevy":
        section += (
            f"The latest stable, non-yanked release resolved on October 21, 2026 was **{version}**. "
            f"Its declared minimum Rust version is {bundle['release']['rust_version']}. "
            "The pinned repository archive exceeds the collector's size limit, so this bundle uses the verified published crate. "
            "The manifests are truncated and source selection is bounded; this is a focused sample, not full engine or migration coverage.\n\n"
        )
    section += (
        f"Install the [checked-in bundle](https://github.com/mcaveniathor/symposiast/tree/main/plugins/{name}) "
        f"with `symposiast install plugins/{name}/bundle`, then run `cargo agents sync` in the consumer workspace. "
        f"Skills and MCP retrieval target {version} exactly. Restart the consuming MCP session after installation.\n\n"
    )
    section += "### Findings used by the generated recipes\n\n"
    for finding in analysis["findings"]:
        section += f"- **{prose(finding['title'])}** ({finding['basis']}): {prose(finding['guidance'])} "
        section += evidence(finding["citations"], sources) + ".\n"
    section += "\n"
    for recipe in analysis["recipes"]:
        rid = recipe["id"]
        check = checks[rid]
        if check["status"] != "compiled":
            raise ValueError(f"{name}/{rid}: verify the recipe before publishing")
        for filename in ["Cargo.toml", "Cargo.lock", "src/main.rs"]:
            files[f"{rid}/{filename}"] = (root / "verification" / rid / filename).read_bytes()
        if files[f"{rid}/src/main.rs"].decode() != recipe["code"]:
            raise ValueError(f"{name}/{rid}: consumer differs from analysis")
        if hashlib.sha256(files[f"{rid}/Cargo.lock"]).hexdigest() != check["lockfile_sha256"]:
            raise ValueError(f"{name}/{rid}: lockfile differs from recorded verification")
        section += f"### {prose(recipe['title'])}\n\n{prose(recipe['description'])}\n\n"
        section += " · ".join(f"[{filename}](examples/{name}/{rid}/{filename})"
                              for filename in ["Cargo.toml", "Cargo.lock", "src/main.rs"]) + "\n\n"
        section += "```toml\n" + files[f"{rid}/Cargo.toml"].decode().rstrip() + "\n```\n\n"
        section += "```rust\n" + recipe["code"].rstrip() + "\n```\n\n"
        section += f"```sh\ncargo check --locked --all-targets --manifest-path site/downloads/{name}/{rid}/Cargo.toml\n```\n\n"
        release = " --release" if name == "bevy" and rid == "profile" else ""
        section += f"Run with `cargo run --locked{release} --manifest-path site/downloads/{name}/{rid}/Cargo.toml`. "
        if name == "axum" and rid != "r11":
            section += "This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe."
        elif name == "bevy" and rid in {"window", "mesh", "profile"}:
            section += "This requires a display and graphics environment. Compilation was verified; graphical runtime behavior was not checked."
        else:
            section += "This is a finite executable."
        section += "\n\nRelease evidence: " + evidence(recipe["citations"], sources) + ".\n\n"
    return files, section, version, len(analysis["recipes"])


def generated():
    files, sections, rows = {}, [], []
    total = 6
    for name in CRATES:
        downloads, section, version, count = generated_crate(name)
        files[name] = downloads
        sections.append(section)
        rows.append(f"| {LABELS[name]} | {version} | {count} | Codex adapter |")
        total += count
    rows.append("| Stucco | 0.3.0 | 6 | Imported analysis |")
    _, stucco = stucco_generated()
    stucco = "## Stucco " + stucco.partition("## Stucco ")[2]
    intro = f'''# Examples you can inspect and run

These {total} recipes compile against exact published crate versions with recorded consumer lockfiles. The versions are validated snapshots. Compilation means `cargo check --locked --all-targets` passed; runtime checks are recorded separately in the [validation notes](https://github.com/mcaveniathor/symposiast/blob/main/docs/validation.md).

| Crate | Version | Recipes | Analysis source |
| --- | --- | --- | --- |
{chr(10).join(rows)}

## Generate recipes from release findings

Symposiast asks its analysis agent to produce findings and complete consumer recipes from the collected evidence. It inventories documentation code blocks, requires an adapted/supporting/skipped decision for each one, validates citations and feature flags, then compiles the generated recipes and renders the plugin. No hand-written recipe input is required.

```sh
symposiast build clap --version 4.6.7 --agent codex \\
  --output .symposiast/clap-generated
symposiast build axum --version 0.8.9 --agent codex \\
  --output .symposiast/axum-generated
symposiast build bevy --agent codex \\
  --output .symposiast/bevy-generated
```

Omit `--version` to select the latest stable, non-yanked release. Generation requires an authenticated agent CLI; a new run may produce different recipes. The checked-in bundles preserve the actual findings, recipes, coverage decisions, and compilation records used by this page. See [agent adapters](adapters.html) for setup and the structured import alternative.

## Use the downloadable projects

Clone the [repository](https://github.com/mcaveniathor/symposiast), or save a recipe's linked `Cargo.toml`, `Cargo.lock`, and `src/main.rs` with the same directory structure. Each recipe includes its verification command. No analysis agent or Symposium installation is needed to compile a saved project. Read its main function before running: a server may listen until stopped, and a graphical Bevy application needs the corresponding display, driver, and platform libraries.

'''
    return files, intro + "".join(sections) + stucco


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    files, guide = generated()
    for name, expected in files.items():
        directory = ROOT / "site/downloads" / name
        if args.check:
            actual = {p.relative_to(directory).as_posix(): p.read_bytes()
                      for p in directory.rglob("*") if p.is_file() and "target" not in p.relative_to(directory).parts}
            if actual != expected:
                parser.exit(1, f"{name} downloads are stale; run python3 scripts/update_examples.py\n")
        else:
            if directory.exists():
                shutil.rmtree(directory)
            for relative, content in expected.items():
                path = directory / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)
    if args.check:
        if GUIDE.read_text() != guide:
            parser.exit(1, "Examples guide is stale; run python3 scripts/update_examples.py\n")
        print("clap, axum, and Bevy downloads and guide match their verified bundles.")
    else:
        GUIDE.write_text(guide)
        print("Updated clap, axum, and Bevy downloads and the examples guide.")


if __name__ == "__main__":
    main()
