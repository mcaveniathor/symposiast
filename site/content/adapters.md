# Agent adapters

Use Claude Code, Codex, or a custom executable to analyze frozen release evidence. You can choose a separate reviewer. Once a bundle exists, MCP retrieval and version checks run locally without an analysis agent.

## Claude Code and Codex

Install and authenticate the selected CLI through its normal workflow. Symposiast inherits the environment needed by that CLI without copying credentials. Both adapters receive the same evidence and analysis instructions.

```sh
symposiast build clap --agent claude-code \
  --output .symposiast/clap

symposiast build axum --agent codex --reviewer claude-code \
  --output .symposiast/axum
```

Either built-in adapter can analyze or review. Codex is the default when `--agent` is omitted. Use `--agent-executable` and `--reviewer-executable` to select particular installations or wrappers:

```sh
symposiast analyze .symposiast/clap \
  --agent claude-code --agent-executable /opt/bin/claude \
  --reviewer codex --reviewer-executable /opt/bin/codex \
  --timeout 900
```

`--timeout` is a positive number of seconds, defaulting to 600. It bounds each agent invocation and each recipe compilation check separately. Sessions are one-shot; Symposiast does not resume a provider session or retry automatically. Reuse `analyze` on the saved evidence to retry deliberately.

Claude Code uses print mode, JSON output, and a response schema. Built-in tools, additional MCP servers, settings-source loading, browser integration, and session persistence are disabled. The result must contain a successful structured JSON response; plain result text is insufficient. Managed settings can still apply.

Codex uses `codex exec` with a read-only sandbox, ephemeral sessions, `--skip-git-repo-check`, a response schema, and a final output file. Progress output is separate from the accepted result. Its runtime state directory must be writable even though its analysis sandbox is read-only.

The built-in CLI interfaces were checked against Claude Code 2.1.303 and Codex 0.159.2. Subprocess contracts are tested with fixtures. Live provider responses were not verified in the initial development environment: Claude Code lacked a login and Codex's injected state directory was read-only. The published clap, axum, and Stucco validation used imported analyses. A working authenticated local CLI is needed to exercise live analysis.

## Import an analysis

An imported JSON file replaces the analysis-agent call. Citation validation, optional review, verification, and rendering still run:

```sh
symposiast collect clap --version 4.6.7 \
  --output .symposiast/clap-import
symposiast analyze .symposiast/clap-import \
  --analysis analysis.json
```

The file must be an Analysis object matching the contract below. Its citations must refer to actual IDs and line ranges in that specific bundle. Do not reuse source IDs from another collection without checking them. `--analysis` also works with `build` when the resulting source identities are known.

## Custom executable contract

Choose `custom` and provide an executable:

```sh
symposiast analyze .symposiast/clap \
  --agent custom --agent-executable ./my-adapter
```

The executable receives no command-line arguments. Use a wrapper if your provider needs arguments or configuration. An explicit relative executable path is resolved before entering the temporary working directory; a bare executable name uses `PATH`.

Standard input contains one complete JSON job, terminated by end of input. The job fields are:

| Field | Meaning |
| --- | --- |
| `schema_version` | Protocol version, currently `2`. |
| `operation` | `analyze` or `review`. |
| `instructions` | Coordinator-provided task and evidence-handling instructions. |
| `release` | Exact crate name and version, checksum, repository metadata, features, and collection timestamp. |
| `warnings` | Collection limitations to consider. |
| `sources` | Source metadata plus the complete frozen text as numbered line objects. |
| `example_inventory` | Extracted documentation examples with source citations, original fence annotations, code, and extraction warnings. |
| `analysis` | `null` for analysis; the proposed Analysis object for review. |
| `output_schema` | The complete response schema for this operation. |

Each entry in `sources` contains `source` metadata (`id`, `kind`, `path`, `url`, `sha256`) and a `lines` array such as `[{"line": 1, "text": "Source text"}]`. Paths describe provenance; they are not files to open from the adapter's temporary working directory. Line numbers are one-based.

Protocol version 2 requires `example_coverage`: one entry per inventoried example, with `example_id`, `disposition` (`adapted`, `supporting`, or `skipped`), `recipe_ids`, and a nonempty `reason`. Used examples must name recipes citing their source lines; skipped examples must name no recipes. An empty inventory uses `[]`. Older bundles gain an inventory when reanalyzed, including through `--analysis`. The inventory covers collected evidence only, and does not imply upstream doctests were run.

For `analyze`, write exactly one Analysis JSON object to standard output. This abbreviated teaching example illustrates the shape; replace its guidance, code, source ID, and lines with evidence-backed content from the actual job:

```json
{
  "summary": "Guidance for the selected release.",
  "example_coverage": [],
  "findings": [
    {
      "id": "parser",
      "title": "Use Parser",
      "guidance": "A recommendation supported by the cited source.",
      "basis": "documented",
      "features": ["derive"],
      "citations": [
        {"source_id": "readme", "start_line": 1, "end_line": 1}
      ]
    }
  ],
  "recipes": [
    {
      "id": "parser",
      "title": "A complete consumer",
      "description": "What this recipe demonstrates.",
      "code": "fn main() {}",
      "default_features": true,
      "features": [],
      "dependencies": {},
      "citations": [
        {"source_id": "readme", "start_line": 1, "end_line": 1}
      ]
    }
  ]
}
```

All fields are required and unknown fields are rejected. `basis` is `documented` or `inferred`. IDs contain 1–100 lowercase letters, digits, hyphens, or underscores, and must be unique within their collection. Citation ranges are inclusive. Every finding and recipe needs valid evidence. An analysis must contain a nonempty summary, at least one finding, and at least one recipe.

Recipe `code` is a complete `src/main.rs`. `features` and `default_features` configure the analyzed crate. Additional registry dependencies use a map:

```json
{
  "tokio": {
    "version": "1",
    "features": ["macros", "rt-multi-thread"],
    "default_features": true
  }
}
```

Path and Git dependencies are unsupported. The built-in adapters use an array of named dependency objects for strict provider schemas, then convert that array to this map. Custom responses and imported analyses use the map directly. Only the verifier assigns compilation results.

For `review`, return an approval with no issues:

```json
{"approved": true, "issues": []}
```

Or return a rejection with at least one nonempty issue:

```json
{"approved": false, "issues": ["The recipe enables an unavailable feature."]}
```

Unresolved review issues stop rendering. A rejection with no issues, or an approval with issues, is invalid.

Return a nonzero exit code on operational failure. Keep logs, progress messages, and Markdown fences out of standard output. Standard error is consumed but not printed verbatim because provider diagnostics can contain private details. Diagnose authentication failures by running the provider CLI directly.

## Evidence and process limits

Jobs run in new temporary directories. Fetched instructions, hooks, and project configuration are treated as source evidence rather than installed as executable configuration. Custom adapters are trusted executables and must apply suitable restrictions to their own providers and subprocesses.

The full job, including numbered evidence and instructions, is limited to 2 MiB. Oversized jobs fail explicitly; the adapter does not silently omit sources. This byte limit is not a model-context guarantee. Chunked analysis and automatic context budgeting are not implemented.

Standard output and result files are limited to 8 MiB; standard error is limited to 1 MiB. Timeouts and output limits terminate the child. On Unix, a dedicated process group also terminates descendants; Windows descendant cleanup is not implemented. Temporary job and result files are removed after completion.

See the upstream [Claude Code CLI reference](https://code.claude.com/docs/en/cli-reference) and [Codex CLI reference](https://developers.openai.com/codex/cli/reference/) for installation-specific behavior.
