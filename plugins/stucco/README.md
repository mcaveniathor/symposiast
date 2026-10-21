# Stucco 0.3.0 plugin

This portable bundle contains release-specific guidance and six runnable recipes for [Stucco](https://github.com/mcaveniathor/stucco): static asset export, Axum fragments, derived tables, validation feedback, server-selected themes, and native status/detail components.

## Install

Install the Symposiast CLI at a durable location, then run from this repository:

```sh
cargo install --path . --locked
symposiast install plugins/stucco/bundle
cd /path/to/consumer
cargo agents sync
```

The consumer must resolve `stucco` **0.3.0** to receive its skills and MCP registration. The SessionStart hook remains eligible for other Stucco versions so it can warn about drift. Configure Symposium for Claude Code and/or Codex first, and honor any existing `SYMPOSIUM_HOME` setting.

Installation validates the frozen source hashes and recorded recipe lockfiles, copies the bundle into the local Symposium plugin home, and creates the skills, MCP server registration, and version hook using your own executable and installation paths. The checked-in bundle contains no machine-specific runtime manifest. Adding this repository's `plugins/` registry enables discovery of the separate Symposiast workflow skill; use the installation command above for the full Stucco plugin.

Restart the consuming MCP session after installation. Its tools are `get_release_info`, `search_guidance`, and `get_recipe`. Inspect the release and features before applying guidance. Symposium 0.5.1 may retain an old MCP registration after a dependency changes; the hook and release metadata identify the mismatch, but stale registration removal may need the consuming tool's configuration.

## Inspect and run

`bundle/bundle.json` records the published crate checksum, repository commit, cited findings, complete recipes, compilation checks, and coverage warnings. `bundle/sources/` contains the frozen evidence. `bundle/verification/` contains the actual consumer projects and lockfiles.

```sh
symposiast check plugins/stucco/bundle --workspace /path/to/consumer
cargo run --locked --manifest-path site/downloads/stucco/static-page/Cargo.toml -- /tmp/stucco-preview
cargo run --locked --manifest-path site/downloads/stucco/axum-fragments/Cargo.toml
```

The static recipe writes a page and its assets for root hosting. Serve `/tmp/stucco-preview` as the document root. The Axum recipe checks full-page, named-fragment, fallback, escaping, and stylesheet responses without opening a listening socket; add `-- --serve` to start the demo server after its checks. The other recipes assert derived table formatting and escaping, invalid form state, an inline dark theme, and semantic status/detail markup.

The release is pinned to repository commit `07335e73dfb22aba2ce89487bcee634a24c2080e`. A fresh collection matched all 31 sources in the existing 0.3.0 analysis. This revision imports refinements with runtime assertions, explicit decisions for all 57 inventoried documentation examples, and two separately hashed upstream license files. It contains 12 findings and six recipes. There was no separate reviewer invocation for the refined analysis; `review` remains null. `generation.json` records that provenance.

Compilation status records `cargo check --locked --all-targets` only. All six executables and their assertions also passed locally, as recorded in the [validation notes](../../docs/validation.md). Browser behavior, application storage, CSRF configuration, and the full component catalog are outside those runtime checks. The generated plugin includes readable and JSON example-coverage reports. Compiler logs normalize the temporary build-directory prefix; input fingerprints and lockfile hashes are unchanged by that normalization.

The [examples guide](../../site/content/examples.md) and all 18 downloadable Stucco files are generated from this bundle. After verifying a changed bundle, refresh them with:

```sh
python3 scripts/update_stucco_examples.py
python3 scripts/update_stucco_examples.py --check
```

## Refresh

Preserve this snapshot when evaluating a newer release:

```sh
symposiast refresh plugins/stucco/bundle --agent codex \
  --output .symposiast/stucco-next
```

Use your configured analysis agent. Review the new bundle and its checks before installing it. Refresh does not upgrade the consumer's dependencies.

## Source licenses

Stucco's frozen source evidence is distributed under its upstream [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) license. The release-specific source URLs and SHA256 hashes are recorded in `bundle.json` inside the bundle.
