# Validation

The initial implementation was checked with Rust 1.99.1 and Symposium 0.5.1.

## Automated checks

- Formatting passes `cargo fmt --all --check`.
- Linting passes `cargo clippy --locked --workspace --all-targets -- -D warnings`.
- All 39 tests pass with `cargo test --locked --workspace --all-targets`.

Tests cover archive checksums and traversal, release selection, bounded evidence selection, real fixture subprocesses for agent adapters, structured responses, timeout cleanup, source citations, stale verification results, transactional replacement, plugin installation, MCP stdio requests, and version/provenance warnings.

## Public-crate integration

The collector resolved clap 4.6.7 and axum 0.8.9 from the crates.io index, verified their published archives, and fetched their pinned repository snapshots. Each bundle contains 48 evidence sources and records coverage limits.

Three recipes per crate were supplied through the structured analysis import path. All six compiled against their exact released crate versions with recorded dependency lockfiles. Additional behavior checks passed:

- clap: six parser tests across derive, subcommand, and fallible-validation recipes.
- axum: three in-memory request examples exercised routing, shared state, and typed JSON, with response assertions and no listening sockets.

These behavior checks are separate from the CLI's recipe verification status, which intentionally records compilation only. Pilot bundles and logs are retained under the ignored `.symposiast/` directory in the development workspace.

The exporter supports both `SYMPOSIUM.toml`, recognized by Symposium 0.5.1, and `Symposium.toml`, used in the current upstream documentation. Both files contain the same manifest.

The official Symposium 0.5.1 recursive validator recognizes the real pilot manifests and skills. In an isolated Symposium configuration, sync installed the clap skill and MCP registration; the registered MCP command returned the compiled recipe. Native SessionStart dispatch for both Claude Code and Codex produced no mismatch warning for the matching release and warned after the consumer resolved clap 4.6.6.

An upstream limitation remains: Symposium 0.5.1 removed the mismatched skill during sync but retained the existing MCP registration. The hook and release metadata identify that mismatch; automatic removal of the stale MCP entry is not established.

## Live agent limitations

Built-in command interfaces were checked against Claude Code 2.1.303 and Codex 0.159.2, and their subprocess contracts are covered by fixture tests. Live provider responses were not verified in this environment:

- Claude Code reports no active login.
- Codex reports an existing login, but its injected state directory is read-only. Startup fails before a provider request because required state files cannot be written.

No credentials were copied or replaced. A normal authenticated CLI with writable runtime state is needed to validate the live analysis path. The successful public-crate integration checks used imported analyses and do not establish live model integration.

## Stucco plugin and examples

Stucco 0.2.1 was resolved as the latest stable release for this run. Its published archive checksum and packaged repository commit `303ee2433ce3c05d0af9c4b79a4e09958d374f92` were verified. The collector retained 27 evidence files. The portable bundle adds `stucco-core/src/bundle.rs` from that exact commit for asset-prefix/export citations, plus the two upstream license texts, for 30 frozen files total. The supplemental files have their own hashes; they are not covered by the facade archive checksum.

Four findings and two complete recipes were supplied through the structured analysis import path. Both passed `cargo check --locked --all-targets`; their recorded lockfiles resolve the Stucco companion crates to 0.2.1. Separate executable assertions passed:

- `static-page`: escaped text, main landmark, skip link, project URL prefix, and exact exported asset bytes.
- `axum-page`: in-memory HTML and stylesheet requests, content types, escaped text, main landmark, immutable asset caching, and exact stylesheet bytes.

The checked-in `plugins/stucco/bundle` preserves the evidence, recipes, compiler results, and lockfiles without runtime manifests or build caches. Compiler logs normalize the temporary build-directory prefix. Its six downloadable Cargo files exactly match the validated consumer projects. CI checks bundle integrity and runs both example executables.

Installation from a relocated bundle produced matching manifest spellings and a valid Symposium 0.5.1 plugin and skill. After deleting the relocated input, installed MCP retrieval still returned Stucco 0.2.1 and the compiled static recipe. Claude project sync registered the skill and MCP. Matching canonical hook input returned no warning; a temporary consumer lockfile changed to 0.2.0 produced the expected mismatch warning. Native Claude Code and Codex SessionStart dispatch also returned neutral responses for the matching release with automatic sync disabled. Codex's global MCP configuration was not modified or exercised in this test. All installed source hashes matched, and the two license texts survived in both the installed bundle and skill evidence.

## Documentation and workflow skill

The Stucco 0.2.1 static site builds with a locked dependency graph. All seven pages and their local assets and anchors pass `site/check_links.py` under both the `/symposiast/` project prefix and a `/` root deployment. The 24 downloadable recipe files match the validated consumer projects exactly, including the recorded lockfile hashes.

Chromium checks cover 1440px, 768px, and 390px layouts with no document overflow, working copy buttons, keyboard skip navigation, and navigation without JavaScript. Automated WCAG A/AA checks pass on all six pages at desktop and phone widths. These are local checks; they do not establish that a GitHub Pages deployment has completed.

The Automation page and expanded navigation also passed layout checks at 1440, 1024, 800, 768, 390, and 320 pixels, with accessibility checks at desktop and phone widths.

## Release guidance action

The reusable action passed 16 local helper tests covering input validation, argument handling, exact-version registry polling, single-attempt generation, failure propagation, source/recipe integrity, archive exclusions, and bootstrap credential filtering. A prefix-only `v` is rejected rather than treated as an omitted version. Actionlint 1.7.7 validated all repository workflows and the release workflow example.

Both the preinstalled-CLI and default-bootstrap paths were exercised with the real Symposiast CLI. The default path installed the pinned Rust toolchain and compiled Symposiast from this checkout with `cargo install --locked`. Both runs collected Stucco 0.2.1, imported the CI fixture analysis, compiled its consumer, and produced a portable archive. Extracting the archive into a fresh directory preserved the evidence hashes; the example's HTML assertions passed and recipient installation created the local Symposium plugin and matching manifest spellings.

The action's CI workflow repeats the imported-analysis integration and uploads the successful archive. These local checks do not establish that a hosted GitHub Actions run, artifact upload, release attachment, or live provider analysis has completed. Provider authentication is configured by the consuming workflow; no provider credentials are bundled with the action or its artifacts.

The `plugins/symposiast` workflow plugin and its `use-symposiast` skill pass Symposium 0.5.1's plugin validator. In an isolated configuration, registering the repository's `plugins/` directory and running `cargo agents use symposiast` installed identical skill contents into both Claude Code's `.claude/skills/` and Codex's `.agents/skills/`. A second sync preserved the installation and `cargo agents status` reported the plugin active. No user-wide agent settings were changed during validation.
