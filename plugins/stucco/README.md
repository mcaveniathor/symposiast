# Stucco 0.2.1 plugin

This portable bundle contains release-specific guidance and two runnable recipes for [Stucco](https://github.com/mcaveniathor/stucco): a static page with exported theme assets, and an Axum page exercised through in-memory requests.

## Install

Install the Symposiast CLI at a durable location, then run from this repository:

```sh
cargo install --path . --locked
symposiast install plugins/stucco/bundle
cd /path/to/consumer
cargo agents sync
```

The consumer must resolve `stucco` **0.2.1** to receive its skills and MCP registration. The SessionStart hook remains eligible for other Stucco versions so it can warn about drift. Configure Symposium for Claude Code and/or Codex first, and honor any existing `SYMPOSIUM_HOME` setting.

Installation validates the frozen source hashes and recorded recipe lockfiles, copies the bundle into the local Symposium plugin home, and creates the skills, MCP server registration, and version hook using your own executable and installation paths. The checked-in bundle contains no machine-specific runtime manifest. Adding this repository's `plugins/` registry enables discovery of the separate Symposiast workflow skill; use the installation command above for the full Stucco plugin.

Restart the consuming MCP session after installation. Its tools are `get_release_info`, `search_guidance`, and `get_recipe`. Inspect the release and features before applying guidance. Symposium 0.5.1 may retain an old MCP registration after a dependency changes; the hook and release metadata identify the mismatch, but stale registration removal may need the consuming tool's configuration.

## Inspect and run

`bundle/bundle.json` records the published crate checksum, repository commit, cited findings, complete recipes, compilation checks, and coverage warnings. `bundle/sources/` contains the frozen evidence. `bundle/verification/` contains the actual consumer projects and lockfiles.

```sh
symposiast check plugins/stucco/bundle --workspace /path/to/consumer
cargo run --locked --manifest-path site/downloads/stucco/static-page/Cargo.toml -- /tmp/stucco-preview/demo
cargo run --locked --manifest-path site/downloads/stucco/axum-page/Cargo.toml
```

The static recipe writes a page and its assets for a `/demo/` URL prefix. Serve `/tmp/stucco-preview` as the document root and use the `/demo/` path. The Axum recipe checks the rendered page and stylesheet without opening a listening socket. Both also appear in the documentation site's examples.

The analysis was supplied through the structured import path, with citations checked against frozen evidence. Compilation status records Cargo checks only; runtime assertions are validated separately. Source selection is bounded. One sibling `stucco-core` API source was added from the same repository commit to document asset-prefix and export APIs; it is identified separately from the published facade archive. Compiler logs replace the temporary build-directory prefix with `<verification-workspace>`; the recorded results and lockfile hashes are unchanged.

## Refresh

Preserve this snapshot when evaluating a newer release:

```sh
symposiast refresh plugins/stucco/bundle --agent codex \
  --output .symposiast/stucco-next
```

Use your configured analysis agent. Review the new bundle and its checks before installing it. Refresh does not upgrade the consumer's dependencies.

## Source licenses

Stucco's frozen source evidence is distributed under its upstream [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) license. The release-specific source URLs and SHA256 hashes are recorded in `bundle.json` inside the bundle.
