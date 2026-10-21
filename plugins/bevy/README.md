# bevy 0.20.0 generated guidance

Symposiast generated 7 findings and 5 complete consumer recipes from the published bevy 0.20.0 release through its authenticated Codex adapter. Recipe code and coverage decisions came from that live invocation, without a hand-written `--analysis` input. All recipes passed `cargo check --locked --all-targets`. No separate reviewer was invoked.

## Inspect

`bundle/bundle.json` contains the release checksum, source hashes, findings, recipes, documentation-example inventory, coverage decisions, and compiler results. `bundle/generation.json` records the generation command and verification environment. The `sources/` and `verification/` directories preserve the evidence and consumer projects, including lockfiles and compiler logs. Temporary build paths in compiler logs are normalized; verification fingerprints and lockfile hashes are unchanged.

The 5 inventoried documentation examples each have an adapted, supporting, or skipped decision with a reason. Coverage applies to the collected evidence, not the entire upstream documentation. Standalone example programs can also support recipes but are not part of the documentation code-block count. Inspect the bundle's collection and extraction warnings before applying guidance.

See the [downloadable examples](../../site/content/examples.md) and [validation notes](../../docs/validation.md) for commands and separate runtime checks.

## Install

Install the Symposiast CLI at a durable location, then run from this checkout:

```sh
symposiast install plugins/bevy/bundle
cd /path/to/consumer
cargo agents sync
```

Skills and MCP retrieval match **bevy 0.20.0** exactly. The version hook warns about other versions. Restart the consuming MCP session after installation. No user-wide installation or workspace synchronization was performed when preparing this bundle.

## Regenerate

Preserve the checked-in snapshot and generate a candidate in a new directory:

```sh
symposiast build bevy --version 0.20.0 --agent codex \
  --output .symposiast/bevy-next
```

Omit `--version` to resolve the latest stable, non-yanked release. The agent may produce different findings and recipes on a new run. Review citations, coverage, feature selection, and compilation before replacing the bundle. Preserve upstream licenses and generation provenance when packaging it. Then regenerate the documentation:

```sh
python3 scripts/update_examples.py
python3 scripts/update_examples.py --check
```

## Source licenses

The frozen upstream evidence is distributed under its [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) license. Source URLs and SHA256 hashes are recorded in `bundle/bundle.json`.

Bevy 0.20.0 was the latest stable, non-yanked release selected on October 21, 2026 and declares Rust 1.97.1 as its minimum. Its repository archive exceeded the collector download limit; evidence comes from the verified published crate. Two manifests are truncated. The graphical recipes compile, but their window, renderer, and profiling output were not run or visually checked. The `hello` and `message` executables passed locally. On Ubuntu, graphical compilation needs development packages including `libasound2-dev`, `libudev-dev`, `libwayland-dev`, and `libxkbcommon-dev`.
