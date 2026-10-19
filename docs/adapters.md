# Agent adapters

The coordinator supports Claude Code, Codex, and custom executable adapters.
Analysis and review are separate jobs; either built-in adapter can fill either
role. A reviewer is optional. Retrieval and version checks use the resulting
bundle locally and do not launch an analysis agent.

Install and authenticate the chosen CLI using its normal installation and login
workflow. Executable paths can be overridden, so wrappers and multiple installed
versions are supported. Authentication and proxy environment variables are
inherited without inspecting or copying credential values.

## Built-in invocation

Codex runs `codex exec` with a read-only sandbox, ephemeral sessions,
`--skip-git-repo-check`, `--output-schema`, and `--output-last-message`.
The job arrives on standard input. Its final JSON is read from the output file;
progress output is not parsed as a result. The adapter never passes permission
bypass flags. The installed CLI must support these options.

Claude Code runs `claude --print --output-format json --json-schema ...`, with
all built-in tools disabled, an empty MCP configuration, settings-source loading
disabled, `dontAsk` permissions, no browser integration, and no session
persistence. Disabling settings sources prevents project and user hooks from
being loaded into the analysis session. Managed settings can still apply.
These flags were checked against the official `@anthropic-ai/claude-code-linux-x64`
2.1.303 package. The adapter requires a successful JSON `result` envelope with
an object in `structured_output`. A text-only `result` is not accepted as a
substitute for structured output.

The two CLIs receive the same evidence and semantic instructions. Built-in
structured output represents each recipe's additional dependencies as an array:

```json
[
  {"name": "tokio", "version": "1", "features": ["macros"], "default_features": true}
]
```

The coordinator converts this array to the bundle's dependency map and rejects
duplicate names. This supports strict response schemas without open-ended object
keys. All built-in schema objects reject additional properties and require every
declared field.

## Custom executable protocol, version 1

Select the custom adapter and provide an executable path. The executable receives
no arguments. Use a wrapper executable when extra arguments or provider-specific
configuration are needed. An explicit relative path such as `./adapter.sh` is
resolved before the subprocess changes directories; a bare executable name uses
`PATH`.

Standard input contains one complete JSON object, terminated by end of input:

```json
{
  "schema_version": 1,
  "operation": "analyze",
  "instructions": "Coordinator-provided task and evidence-handling instructions",
  "release": {
    "name": "clap",
    "version": "4.5.0",
    "checksum": "published crate checksum",
    "repository": null,
    "repository_commit": null,
    "rust_version": null,
    "features": {"derive": []},
    "collected_at": "collection timestamp"
  },
  "warnings": [],
  "sources": [
    {
      "source": {
        "id": "readme",
        "kind": "readme",
        "path": "sources/README.md",
        "url": "https://example.invalid/README.md",
        "sha256": "frozen source checksum"
      },
      "lines": [
        {"line": 1, "text": "Use the Parser trait."}
      ]
    }
  ],
  "analysis": null,
  "output_schema": {}
}
```

The example abbreviates `instructions` and `output_schema`; actual jobs contain
the full instructions and response schema. Every frozen source is included in
full, with its original ID and one-based line numbers. The schema is the contract
for the requested operation. The `path` is provenance metadata, not a file the
adapter should read from its working directory.

For `analyze`, standard output must contain exactly one Analysis JSON object:

```json
{
  "summary": "Release-specific guidance",
  "findings": [
    {
      "id": "parser",
      "title": "Use Parser",
      "guidance": "A recommendation supported by the cited evidence.",
      "basis": "documented",
      "features": ["derive"],
      "citations": [{"source_id": "readme", "start_line": 1, "end_line": 1}]
    }
  ],
  "recipes": [
    {
      "id": "parser",
      "title": "A minimal consumer",
      "description": "What the recipe demonstrates.",
      "code": "fn main() {}",
      "default_features": true,
      "features": [],
      "dependencies": {},
      "citations": [{"source_id": "readme", "start_line": 1, "end_line": 1}]
    }
  ]
}
```

The recipe code above only illustrates the protocol. Real recipes must implement
the described task using the selected release. `basis` is `documented` or
`inferred`. Citation ranges are inclusive. Additional dependencies use a map from
crate name to `{ "version": "1", "features": [], "default_features": true }`.
All fields are required and unknown fields are rejected. Findings and recipes
must have evidence and valid, unique IDs. The coordinator validates source
checksums, citation ranges, and dependency declarations before accepting the
analysis. Compiler checks happen separately; agents cannot assign verification
results.

For `review`, the input includes the proposed Analysis object in `analysis` and
the output must be:

```json
{"approved": true, "issues": []}
```

A rejection uses `approved: false` and one or more nonempty issue strings.
An approval with issues, or a rejection with no issues, is invalid.

Return a nonzero exit status on operational failure. Do not mix logs, progress,
or Markdown fences into standard output. Standard error is drained but not
reported verbatim, because provider diagnostics can contain private details.
Investigate authentication or provider errors by running the CLI directly.

## Execution boundaries and limits

Jobs run in a new temporary directory outside the collected sources. Fetched
`AGENTS.md`, `CLAUDE.md`, repository hooks, and project configurations are never
installed as instructions. Source contents are framed as untrusted evidence.
Custom adapters are trusted executables selected by the user and are responsible
for applying suitable restrictions to their own providers and child processes.

The complete input, including instructions and numbered evidence, is limited to
2 MiB. Oversized evidence fails explicitly; it is never silently truncated or
sampled. This byte limit bounds transport and memory use, not model context usage.
A provider can have a smaller context window and reject a job within the limit.
Chunked analysis and context budgeting are not implemented yet.

Standard output and final result files are limited to 8 MiB; standard error is
limited to 1 MiB. All streams are handled concurrently to avoid subprocess pipe
deadlocks. A positive wall-clock timeout covers input delivery, output collection,
and process exit. Timeout, output overflow, or cancellation terminates the direct
child; on Unix, a dedicated process group also terminates descendants. Temporary
job and result files are deleted after completion. Windows descendant process
cleanup is not implemented.

The initial adapters use one-shot sessions. They do not resume a provider session
or retry automatically, so unexpected model costs are not multiplied silently.
The saved evidence bundle can be reused for a fresh analysis job after a failure.

References: [Codex CLI reference](https://developers.openai.com/codex/cli/reference/)
and [Claude Code CLI reference](https://code.claude.com/docs/en/cli-reference).
