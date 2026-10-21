# CLI reference

Run `symposiast --help` or `symposiast <command> --help` for local usage. Bundle paths name directories containing `bundle.json` and its frozen sources.

## Commands

| Command | Purpose |
| --- | --- |
| `collect <name> --output <directory>` | Resolve a release and collect evidence without an agent. |
| `build <name> --output <directory>` | Collect, analyze, verify recipes, and render a plugin. |
| `analyze <bundle>` | Analyze existing evidence, verify recipes, and render. |
| `verify <bundle>` | Recheck recipes and update the rendered plugin. |
| `render <bundle>` | Validate a bundle and regenerate its plugin. |
| `install <bundle>` | Copy a durable plugin into the local Symposium home. |
| `serve <bundle>` | Serve local MCP retrieval over stdio. |
| `check <bundle>` | Report dependency-version and source mismatches. |
| `hook <bundle>` | Handle a canonical Symposium SessionStart event. |
| `refresh <bundle> --output <directory>` | Analyze the latest release in a separate bundle. |

### Collection and build

`collect` and `build` accept `--version <exact-version>`. If omitted, they choose the latest stable, non-yanked release. An explicit version may select a prerelease; yanked releases remain excluded. `--output` is required and must point to a new or empty directory.

`build` and `analyze` share these options:

| Option | Behavior |
| --- | --- |
| `--agent <kind>` | `codex` (default), `claude-code`, or `custom`. |
| `--agent-executable <path>` | Override the executable; required for `custom`. |
| `--reviewer <kind>` | Run an optional separate review with any adapter kind. |
| `--reviewer-executable <path>` | Override reviewer executable; requires `--reviewer`. |
| `--analysis <file>` | Import Analysis JSON instead of invoking the analysis agent. |
| `--timeout <seconds>` | Positive per-invocation and per-recipe timeout; default `600`. |
| `--skip-verify` | Render recipes with explicitly unverified status without running Cargo. |

See [agent adapters](adapters.html) for authentication, the custom protocol, and JSON imports. Failed analysis preserves the previous bundle and reports the retained staged run.

### Verify and render

```sh
symposiast verify .symposiast/clap --timeout 300
symposiast verify .symposiast/clap --offline
symposiast render .symposiast/clap
```

`verify` defaults to a 300-second timeout per recipe. `--offline` prevents Cargo network access and requires locally cached dependencies. Each consumer project pins the analyzed crate exactly, records a lockfile, and runs `cargo check --locked --all-targets`.

Compilation results include a fingerprint of the release and recipe inputs, the compiler version, log path, and available lockfile checksum. Editing inputs invalidates a previous compiled claim until verification is rerun. `render` validates evidence and existing checks but does not invoke Cargo or an agent. Unresolved review issues prevent rendering.

### Installation

```sh
symposiast install .symposiast/clap
symposiast install .symposiast/clap --symposium-home /path/to/symposium
```

The home directory is selected in order: `--symposium-home`, `SYMPOSIUM_HOME`, then `~/.symposium`. Installation copies the bundle under `plugins/<crate>-<version>`, retains evidence and verification artifacts, and omits Cargo target caches. It records the currently running Symposiast executable's absolute path.

Run `cargo agents sync` in the consuming workspace after installation. The executable must stay at the recorded location. Symposium's manifest registration does not install the Symposiast binary for you.

### Refresh

```sh
symposiast refresh .symposiast/clap \
  --agent codex --output .symposiast/clap-next
```

`refresh` accepts `--agent`, `--agent-executable`, and `--timeout` with the same defaults as `build`. It always resolves the latest stable release and performs verification. It does not accept `--version`, `--reviewer`, `--analysis`, or `--skip-verify`; use separate `collect` and `analyze` steps for those choices.

The output must be separate from the existing bundle. Refresh does not upgrade consumer dependencies, install the new plugin, or restart an MCP session. Review and install the new bundle, sync from the consumer project, then restart the consuming MCP session.

## MCP retrieval

```sh
symposiast serve .symposiast/clap
```

The server reads newline-delimited JSON-RPC from stdin and writes protocol responses to stdout. Normal use is through the generated Symposium MCP registration. Retrieval needs neither network access nor model credentials.

| Tool | Arguments | Result |
| --- | --- | --- |
| `search_guidance` | `query`: 1–512 characters; optional `limit`: 1–20, default `5`. | Matching findings and recipe summaries with release context, citations, and available check status. |
| `get_recipe` | `id`: a recipe identifier. | Complete Rust code, features, dependencies, evidence, and compilation result. |
| `get_release_info` | No arguments. | Exact release, source provenance, summary, review, verification, and coverage warnings. |

Search matches words against titles, guidance or descriptions, and feature names. It does not use embeddings or invoke a model. Results include the exact release they describe; inspect that version before applying a recipe.

A `tools/call` request has this shape:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "search_guidance",
    "arguments": {"query": "derive parser", "limit": 3}
  }
}
```

Each request must occupy one line on the stdio transport. MCP clients handle initialization and message framing. Tool results contain JSON inside a text content block. Requests are limited to 1 MiB. Citation excerpts may be shortened to 2,048 characters, with an explicit truncation marker; the frozen source remains in the bundle.

## Version checks and hooks

```sh
symposiast check .symposiast/clap --workspace /path/to/consumer
```

`check` defaults to the current directory and searches ancestors for the nearest `Cargo.lock`. It prints a JSON array of advisory warnings. It does not resolve dependencies or contact the registry, and version mismatch warnings do not make the command fail.

The check reports unsupported versions, partial coverage when several versions are present, and source mismatches for Git, path, alternate-registry, or conflicting-checksum packages. The analyzed crate's absence or a missing lockfile produces no warning; an empty result alone does not establish that the bundle matches a resolved dependency.

The generated SessionStart hook receives Symposium's canonical event:

```json
{"SessionStart":{"cwd":"/path/to/consumer"}}
```

When a mismatch is found, `symposiast hook <bundle>` returns context in the canonical response:

```json
{"SessionStart":{"additionalContext":"Guidance version mismatch: ..."}}
```

No warning returns `{}`. Invalid input or unavailable checks also return `{}` and report diagnostics on stderr, keeping the hook nonblocking. Symposium translates supported integrations' native hook events into this canonical format.

Skills and MCP registration use exact-version predicates. The broader crate predicate keeps the hook eligible after upgrades. Symposium 0.5.1 can retain a previously registered MCP server when its predicate no longer matches; remove stale entries through the consuming tool's configuration as needed. The hook and returned release metadata help identify this drift, but they do not automatically unregister old servers.

## Collection and validation boundaries

The collector verifies the published crates.io archive checksum and anchors optional GitHub repository evidence to the packaged VCS commit. It never substitutes repository HEAD for missing release evidence. It collects source rustdoc, metadata, documentation, changelogs, tests, and examples; arbitrary documentation-site crawling is not implemented.

Selection is bounded to 48 sources, 48 KiB per file, and 320 KiB of retained evidence. Download and expanded-archive limits are 32 MiB and 192 MiB. Missing repository material, truncation, and omitted sources appear in bundle warnings. Read those warnings when assessing coverage.

Validation compiled twelve pinned clap, axum, and Stucco recipes. Additional parser tests, rendering checks, and in-memory HTTP assertions were run separately; the CLI's `compiled` status records only Cargo compilation. These examples are documented in [examples](examples.html). The automated suite covers collection, adapters, citations, verification, installation, MCP, and version warnings without requiring provider credentials.

## Registry sharing and requests

Browse the [Symposiast Registry](https://mcaveniathor.github.io/symposiast-registry/) for exact-release skills, recipes, evidence, and compilation status. [Install guidance](https://mcaveniathor.github.io/symposiast-registry/subscribe.html) through Symposium, or read the [export guide](https://github.com/mcaveniathor/symposiast/blob/main/docs/registry.md) to produce a portable entry.

To share a verified bundle, follow the [submission guide](https://github.com/mcaveniathor/symposiast-registry/blob/main/CONTRIBUTING.md). To ask for a crate/release or report a correction, use the registry’s [issue forms](https://github.com/mcaveniathor/symposiast-registry/issues/new/choose). Requests need no bundle or agent credentials.

This is a community registry with an explicit subscription, separate from Symposium's official recommendations. The [maintainer guide](maintainers.html) explains their separate review processes, crate-embedded publishing limitations, and how to try guidance with consumers. [Usage reports](https://github.com/mcaveniathor/symposiast-registry/issues/new?template=usage-feedback.yml) distinguish installation, agent consultation, task outcome, and repeat use.
