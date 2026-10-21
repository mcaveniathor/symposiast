# Getting started

Turn a Rust crate release into a local Symposium plugin with source-backed guidance, complete consumer recipes, MCP retrieval, and a dependency-version hook.

## Start with shared guidance

Check the [Symposiast Registry](https://mcaveniathor.github.io/symposiast-registry/) for your exact crate release before generating a new bundle. Its portable skills include recipes, frozen evidence, and compilation records. Follow the [registry installation guide](https://mcaveniathor.github.io/symposiast-registry/subscribe.html) to subscribe through Symposium; using these entries does not require the Symposiast CLI or an analysis-agent login.

Inspect the entry's features, source identity, collection warnings, and review status before applying it. Portable entries supply skills and references. For MCP retrieval and version hooks, use a full bundle installed with Symposiast, as described below.

If coverage is missing, [request a crate or release](https://github.com/mcaveniathor/symposiast-registry/issues/new?template=guidance-request.yml), or generate guidance with this guide and [submit a verified entry](https://mcaveniathor.github.io/symposiast-registry/contributing.html).

## Install the CLI

Install Rust with [rustup](https://rustup.rs/), then build Symposiast from source:

```sh
git clone https://github.com/mcaveniathor/symposiast.git
cd symposiast
cargo install --path . --locked
symposiast --help
```

The checkout pins its development toolchain in `rust-toolchain.toml`. Keep the installed binary at a durable location: installed plugins record its absolute path. Ensure Cargo's binary directory is on your `PATH`.

For analysis, install and authenticate [Claude Code](https://code.claude.com/docs/en/setup) or [Codex](https://developers.openai.com/codex/cli/). Symposiast uses their existing CLI authentication. See [agent adapters](adapters.html) for custom executables and importing an analysis without calling a provider.

Install and configure [Symposium](https://github.com/symposium-dev/symposium) to distribute the plugin to your coding tools. Cargo and rustc must remain available for recipe verification.

## Build your first plugin

Choose a crate and an analysis agent:

```sh
symposiast build clap --agent claude-code \
  --output .symposiast/clap
```

Or use Codex with an optional Claude Code review:

```sh
symposiast build axum --agent codex --reviewer claude-code \
  --output .symposiast/axum
```

The output directory must be new or empty. Without `--version`, each build resolves the latest stable, non-yanked release available from crates.io at that time. To target your application's resolved dependency, pass an exact version:

```sh
symposiast build clap --version 4.6.7 --agent codex \
  --output .symposiast/clap-4.6.7
```

Here `4.6.7` is a pinned example, not a promise that it remains the latest release. Prereleases require an explicit `--version`.

The build collects and verifies the published archive, gathers repository evidence at its packaged commit when available, requests structured guidance, validates citations, checks each consumer recipe with Cargo, and renders the plugin. An optional review must approve the analysis before rendering can continue.

## Inspect the result

The bundle is an inspectable directory:

- `bundle.json` records the exact release, archive checksum, sources, findings, recipes, review, checks, and coverage warnings.
- `sources/` contains the frozen evidence cited by the guidance.
- `verification/` contains consumer projects, dependency lockfiles, and compiler logs.
- `plugin/` contains skills, references, MCP registration, and the version-check hook.

Both `plugin/SYMPOSIUM.toml` and `plugin/Symposium.toml` are emitted with identical contents for released and current upstream naming conventions.

Review the coverage warnings and recipe checks before using the guidance. A compiled recipe passed `cargo check --locked --all-targets`; that status does not assert runtime behavior or prove a recommendation is idiomatic. Findings identify documented advice separately from inferred patterns. The [examples](examples.html) show the pinned clap, axum, and Stucco recipes used for validation.

## Install into your project

Install the completed bundle, then sync inside the project that uses the crate:

```sh
symposiast install .symposiast/clap
cd /path/to/consumer
cargo agents sync
```

The default plugin home is `~/.symposium`. Set `SYMPOSIUM_HOME` or pass `install --symposium-home /path/to/home` for a separate installation. Installation copies the evidence and consumer artifacts into a durable local bundle; it does not depend on the original working directory remaining available.

Skills and MCP registration target the exact analyzed crate version. The SessionStart hook remains eligible when another version of the crate is present so it can warn about drift. Retrieval and version checks operate locally and do not call an analysis agent.

Symposium 0.5.1 can leave an existing MCP registration after a dependency stops matching. Check the release in retrieved results and remove stale registrations through the consuming tool's configuration when necessary. Hook support depends on the consuming integration.

## Teach your agent the workflow

The repository includes a [use-symposiast skill](https://github.com/mcaveniathor/symposiast/blob/main/plugins/symposiast/skills/use-symposiast/SKILL.md) that guides agents through Symposiast and Symposium together. It starts with the project's resolved crate version, reuses matching guidance, and only runs analysis when new evidence or guidance is needed.

Add the checkout as a registry in your existing Symposium configuration (`~/.symposium/config.toml`, or the configuration under `SYMPOSIUM_HOME`):

```toml
[[registry]]
name = "symposiast"
path = "/absolute/path/to/symposiast/plugins"
```

Then enable the skill in the consuming project:

```sh
cargo agents use symposiast
cargo agents sync
```

Symposium distributes it to your configured tools. Reload the session if needed, then ask your agent to use `use-symposiast` for the crate you are working with. This is an instruction skill; installing it does not run analysis or start a server. See [skill installation details](https://github.com/mcaveniathor/symposiast/blob/main/plugins/README.md) for global enablement and direct Claude Code/Codex installation.

## Reuse evidence and update guidance

Collect first when you want to inspect the evidence before analysis:

```sh
symposiast collect clap --output .symposiast/clap-research
symposiast analyze .symposiast/clap-research --agent codex
```

If analysis fails, the collected evidence remains available for another attempt. Analysis and verification stage their updates, preserving the previous bundle when a run fails. The error reports the retained failed run for inspection.

Refresh into a separate directory when you want guidance for the latest release:

```sh
symposiast refresh .symposiast/clap --agent codex \
  --output .symposiast/clap-next
symposiast install .symposiast/clap-next
```

Refresh does not upgrade your application's dependencies. Review the new bundle, sync from the consumer workspace, and restart the consuming MCP session after installation. See the [CLI reference](reference.html) for verification, direct retrieval, and version checks.

## Current scope

Collection covers public crates.io packages and commit-pinned GitHub repository snapshots: source rustdoc, metadata, documentation, changelogs, tests, and examples. It does not crawl arbitrary documentation websites. Evidence selection has explicit limits and records omissions. clap, axum, and Stucco have validated recipes.

Cargo can execute dependency build scripts and procedural macros during checks. Verification limits process lifetime and inherited environment variables, but does not provide an operating-system sandbox. Use a disposable container or VM when checking untrusted packages.
