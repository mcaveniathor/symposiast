# Validation

The initial implementation was checked with Rust 1.99.1 and Symposium 0.5.1.

## Automated checks

- Formatting passes `cargo fmt --all --check`.
- Linting passes `cargo clippy --locked --all-targets -- -D warnings`.
- All 39 tests pass with `cargo test --locked --all-targets`.

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
