# Use Symposiast with Symposium

The [`use-symposiast`](symposiast/skills/use-symposiast/SKILL.md) skill teaches an agent how to collect release evidence, generate and inspect crate guidance, install a bundle, and use Symposium to make its skills, MCP retrieval, and version hook available in the consuming project.

It starts with the project's resolved dependency version and existing guidance. It does not interpret “latest” as permission to upgrade the project or repeatedly invoke analysis when a matching bundle already exists.

## Install through Symposium

Install the Symposiast CLI from this checkout and configure Symposium for your tools. With Symposium 0.5.1, `cargo agents init --add-agent claude --add-agent codex` adds both supported integrations through its normal setup flow.

Add this entry to your existing Symposium configuration (`~/.symposium/config.toml`, or the configuration under `SYMPOSIUM_HOME`), preserving its other entries:

```toml
[[registry]]
name = "symposiast"
path = "/absolute/path/to/symposiast/plugins"
```

Use the absolute path to the `plugins/` directory in a durable checkout of this repository. Then, inside the Rust project where you want the workflow available:

```sh
cargo agents use symposiast
cargo agents sync
cargo agents status
```

The plugin is enabled explicitly by name; the consumer does not need a Cargo dependency on the Symposiast binary. To enable it in every workspace, use `cargo agents use symposiast --global` instead. The repository supplies both manifest filename spellings for released and current Symposium conventions.

Symposium installs the skill into each configured integration's skill directory. Reload the agent session if newly installed skills are not immediately discovered, then ask it to use `use-symposiast` for the relevant crate. This workflow skill contains instructions only: it does not start an MCP server or execute an analysis during installation.

## Direct skill installation

For a tool that accepts standard `SKILL.md` directories, copy the complete `use-symposiast/` directory into the project's `.agents/skills/` for Codex or `.claude/skills/` for Claude Code. Preserve any existing directory of that name rather than overwriting it without review. Symposium distribution is preferred when using both tools.

## Validate the skill

From the repository root:

```sh
cargo agents plugin validate plugins --no-check-crates --json
```

The skill's instructions distinguish compilation checks from runtime tests, require version and feature alignment before applying recipes, and explain the Symposium 0.5.1 stale MCP registration limitation.
