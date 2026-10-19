# Symposiast

Build release-specific Rust crate guidance for the [Symposium](https://github.com/symposium-dev/symposium) ecosystem. Given a crate, Symposiast collects its published source and pinned repository evidence, asks a chosen agent to produce practical guidance, checks consumer examples, and exports skills, MCP retrieval, and a dependency-version hook.

Claude Code and Codex are the initial agent integrations. A custom executable adapter can connect other agents. The first target crates are clap and axum; Bevy-specific workflows are planned.

## Install

Install Rust through rustup, then build the CLI:

```sh
cargo install --path . --locked
symposiast --help
```

The repository pins its development toolchain in `rust-toolchain.toml`. Analysis requires an installed, authenticated agent CLI; authentication stays with that CLI. Cargo and rustc must be available to compile recipes. Symposium is required to distribute the resulting plugin into supported coding tools.

## Build a plugin

```sh
symposiast build clap --agent claude-code --reviewer codex \
  --output .symposiast/clap

symposiast build axum --agent codex \
  --output .symposiast/axum
```

Omit `--reviewer` to use one agent. `--agent-executable` and `--reviewer-executable` override executable locations. Use `--version X.Y.Z` for an exact release; the default is the latest stable, non-yanked version. The output directory must be new or empty.

Each run resolves and verifies a crate archive, gathers bounded release evidence, analyzes it, validates source citations, compiles complete consumer examples, and renders `plugin/Symposium.toml` and supporting skills. Findings distinguish documented advice from inferred patterns. Compilation status records the exact recipe inputs, toolchain, and consumer lockfile; it does not claim runtime testing.

Analysis and verification run in a staged bundle. If a step fails, the previous bundle remains intact and the error identifies the retained failed run for inspection. `--timeout` bounds each agent invocation and each recipe check. `--skip-verify` produces explicitly unverified recipes.

## Inspect, retry, and refresh

Collection and analysis can be separate:

```sh
symposiast collect clap --output .symposiast/clap
symposiast analyze .symposiast/clap --agent codex
symposiast verify .symposiast/clap
```

`analyze --analysis path/to/analysis.json` imports a structured analysis instead of invoking an agent. It still validates citations and verifies recipes. This is useful for reviewing or repairing findings and for automated integrations. The structure is defined in [the adapter contract](docs/adapters.md).

```sh
symposiast refresh .symposiast/clap --agent codex \
  --output .symposiast/clap-next
```

Refresh resolves the latest release into a separate bundle. It does not change a consumer's dependencies or replace installed guidance automatically. Reinstall the new bundle and restart the consuming MCP session after a refresh.

## Install into Symposium

```sh
symposiast install .symposiast/clap
cd /path/to/consumer
cargo agents sync
```

Installation copies the bundle into the local Symposium plugin source and records the current executable's absolute path. Install the CLI at a durable location before installing plugins. Set `--symposium-home` or `SYMPOSIUM_HOME` to use a separate installation directory. The default is `~/.symposium`.

Skills and MCP registration match the exact analyzed crate version. The version-check hook remains eligible for other versions of that crate so it can report drift. Checks use `Cargo.lock`, including multiple installed versions, and are advisory. A missing lockfile cannot establish the resolved version. Symposium/agent support for hooks varies.

Symposium 0.5.1 can retain an existing MCP registration after a dependency stops matching its predicate. The hook warns about the mismatch and MCP results identify their exact release, but the old registration may need removal through the consuming tool's configuration. Do not apply old guidance to the changed dependency.

Symposium's current MCP manifest registers a command but does not install that binary. The generated plugin therefore uses the existing Symposiast executable.

## Local retrieval and version checks

```sh
symposiast serve .symposiast/clap
symposiast check .symposiast/clap --workspace /path/to/consumer
```

`serve` uses MCP JSON-RPC over stdio. Its tools are `search_guidance`, `get_recipe`, and `get_release_info`; results include version context, source references, and available compilation results. Retrieval requires neither network access nor an analysis agent. `hook` accepts Symposium's canonical SessionStart event on stdin and returns nonblocking version guidance.

## Scope and execution

The collector supports public crates.io packages and commit-pinned GitHub repository material. It reads source rustdoc, package metadata, documentation, changelogs, tests, and examples. It does not crawl arbitrary documentation sites. Collection limits and missing sources are recorded in `bundle.json`; inspect those warnings before treating the guidance as comprehensive. Prereleases require an explicit version.

Examples run as Cargo consumer projects. Cargo can execute dependency build scripts and procedural macros. The verifier limits inherited environment variables and process lifetime, but is not a container sandbox. Use a disposable container or VM when verifying untrusted packages. Agent adapters likewise rely on each agent's supported execution controls.

## Development

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

The automated suite uses local evidence and subprocess fixtures; it does not require model credentials. Live model calls and public-registry collection are separate integration checks.

See [the design](docs/design.md), [agent adapter contract](docs/adapters.md), and [validation results](docs/validation.md).
