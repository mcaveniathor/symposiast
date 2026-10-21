# clap 4.6.7 generated guidance

Symposiast generated 13 findings and 8 complete consumer recipes from the published clap 4.6.7 release through its authenticated Codex adapter. Recipe code and coverage decisions came from that live invocation, without a hand-written `--analysis` input. All recipes passed `cargo check --locked --all-targets`. No separate reviewer was invoked.

## Inspect

`bundle/bundle.json` contains the release checksum, source hashes, findings, recipes, documentation-example inventory, coverage decisions, and compiler results. `bundle/generation.json` records the generation command and verification environment. The `sources/` and `verification/` directories preserve the evidence and consumer projects, including lockfiles and compiler logs. Temporary build paths in compiler logs are normalized; verification fingerprints and lockfile hashes are unchanged.

The 122 inventoried documentation examples each have an adapted, supporting, or skipped decision with a reason. Coverage applies to the collected evidence, not the entire upstream documentation. Standalone example programs can also support recipes but are not part of the documentation code-block count. Inspect the bundle's collection and extraction warnings before applying guidance.

See the [downloadable examples](../../site/content/examples.md) and [validation notes](../../docs/validation.md) for commands and separate runtime checks.

## Install

Install the Symposiast CLI at a durable location, then run from this checkout:

```sh
symposiast install plugins/clap/bundle
cd /path/to/consumer
cargo agents sync
```

Skills and MCP retrieval match **clap 4.6.7** exactly. The version hook warns about other versions. Restart the consuming MCP session after installation. No user-wide installation or workspace synchronization was performed when preparing this bundle.

## Regenerate

Preserve the checked-in snapshot and generate a candidate in a new directory:

```sh
symposiast build clap --version 4.6.7 --agent codex \
  --output .symposiast/clap-next
```

Omit `--version` to resolve the latest stable, non-yanked release. The agent may produce different findings and recipes on a new run. Review citations, coverage, feature selection, and compilation before replacing the bundle. Preserve upstream licenses and generation provenance when packaging it. Then regenerate the documentation:

```sh
python3 scripts/update_examples.py
python3 scripts/update_examples.py --check
```

## Source licenses

The frozen upstream evidence is distributed under its [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) license. Source URLs and SHA256 hashes are recorded in `bundle/bundle.json`.
