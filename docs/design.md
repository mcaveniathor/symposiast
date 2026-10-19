# Design

Symposiast builds release-specific tooling for developers using Rust crates. Validated examples cover clap, axum, and Stucco.

## Release and evidence

A run resolves the latest stable, non-yanked release unless given an exact version. The published archive and its registry checksum identify the release. Repository material is anchored to the packaged VCS revision when available. Repository HEAD is not substituted for missing release evidence.

The version 1 bundle records release metadata, source hashes, source locations, findings, complete consumer recipes, review outcomes, and compilation checks. Source references identify actual lines in the frozen evidence. Gathering and model context have explicit limits; omissions and missing repository sources are reported.

Documented and inferred recommendations are separate. Compiled status comes only from the verification process, not from an analysis response. A successful compile does not establish idiomaticity or runtime behavior.

## Agents

Claude Code and Codex are the initial built-in integrations. A custom executable can implement the structured adapter contract. Users retain their existing CLI authentication. One agent can produce the analysis; a second can review it. Unresolved review issues prevent automatic rendering.

The coordinator supplies the frozen evidence and validates returned structure and citations. Source contents are research data, not authority to run commands or change permissions. Adapters run in temporary directories. Their native execution restrictions do not constitute a universal sandbox.

Analysis output is nondeterministic. Recorded sources, structured findings, generated consumer projects, dependency lockfiles, and toolchain versions make the result inspectable and its compilation checks reproducible.

## Verification

Each recipe is a complete Rust `main.rs` with an explicit feature selection and optional registry dependencies. The coordinator generates a separate Cargo project pinned to the analyzed crate version and checks all targets with a lockfile. Additional dependency versions are resolved into that consumer lockfile.

Cargo checks execute build scripts and procedural macros. The verifier limits inherited environment variables and bounds execution time, but does not provide an operating-system sandbox. Run verification in a disposable container or VM for untrusted dependencies. Runtime behavior and unsupported platforms remain outside a compilation result.

## Symposium outputs

Every completed bundle renders a Symposium manifest, skills with supporting references, an MCP registration, and a SessionStart version-check hook. Skills and retrieval activate for the exact analyzed version. The hook uses a broader crate predicate so it can detect version drift.

MCP retrieval and version checks are local and deterministic. They do not invoke an analysis agent. Symposium installs skills and registers the runtime; the Symposiast executable must already be present. Installation copies the bundle to a durable local plugin directory and records an absolute executable path.

## Deliberate boundaries

- Public crates.io packages and pinned GitHub repository snapshots are the first collection sources. Arbitrary documentation-site crawling is not included.
- Source selection is bounded and reports omissions. An evidence archive is not a guarantee that every API was covered.
- Exact-version predicates establish the initial compatibility scope. Broader semver compatibility needs separate validation.
- Review disagreements stop the pipeline; there is no majority vote that overrides evidence.
- Refresh creates a separate bundle. It does not silently upgrade a consumer dependency or overwrite human edits.
