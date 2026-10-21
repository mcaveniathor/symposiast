# The Symposiast registry

Browse the [registry website](https://mcaveniathor.github.io/symposiast-registry/) for available releases, installation instructions, and contribution routes. The [registry repository](https://github.com/mcaveniathor/symposiast-registry) is the source of truth for entries and requests.

`export-registry` turns a verified bundle into a portable Symposium plugin containing skills, recipes, and their evidence. Recipients need Symposium to sync the skills; they do not need Symposiast or an analysis-agent login.

```sh
symposiast export-registry plugins/stucco/bundle \
  --output .symposiast/registry/stucco-0-2-1
cargo agents plugin validate .symposiast/registry --no-check-crates
```

The output is one plugin directory, not the registry root. Put multiple exported entries alongside one another. Both `Symposium.toml` and `SYMPOSIUM.toml` are emitted for compatibility with Symposium 0.5.1 and the current documentation.

The command reads the source bundle without rendering or changing it. It refuses an existing output path, including an empty directory or symlink. Export a revision into a fresh directory, review its diff against the published entry, and replace that entry through the registry's normal PR process. Older crate releases should remain separate entries.

## What gets shared

The plugin activates only for the exact crate release. Its skill contains:

- Findings and complete recipe code, with feature and dependency declarations.
- All frozen evidence, including any license sources recorded in the bundle.
- `references/provenance.json`: release metadata, the source bundle JSON hash, generator version, evidence URLs and hashes, review status, compilation fingerprints, and lockfile hashes.
- Recorded dependency lockfiles under `references/lockfiles/`.

Provenance paths are relative to `references/`. The files stay together when Symposium installs the skill. Review status remains `null` for imported analyses without a recorded review; export does not imply agent or maintainer approval. Compilation records preserve the recorded rustc version, but the current bundle schema does not record the verification platform. Compilation does not establish runtime behavior.

Every recipe must have a current successful compilation record and a recorded lockfile. Invalid evidence, stale fingerprints, changed lockfiles, missing compilation results, and unresolved review issues prevent export. The command does not invoke an agent, compile recipes, or contact a registry; run `symposiast verify` to refresh compilation first.

This export includes no hooks, MCP registration, executable installation, or local runtime paths. To use those capabilities, obtain the full bundle and run `symposiast install` locally. Use one distribution of a release's skill in a workspace; the local runtime plugin and portable entry intentionally share the skill identity.

## Subscribe locally or through Git

Add the containing registry directory to your existing Symposium configuration:

```toml
[[registry]]
name = "symposiast"
path = "/absolute/path/to/registry"
```

The public [Symposiast registry](https://github.com/mcaveniathor/symposiast-registry) contains the Stucco pilot. To subscribe, replace `path` with its Git URL:

```toml
[[registry]]
name = "symposiast"
git = "https://github.com/mcaveniathor/symposiast-registry"
```

Configure your agents with Symposium, then run `cargo agents sync` in the consuming Rust workspace. Confirm the installed skills with `cargo agents status`. Version selection follows Symposium's dependency matching; always check the consumer's actual version and source before applying a recipe.

See Symposium's [registry configuration](https://symposium.dev/reference/configuration.html) and [discovery rules](https://symposium.dev/reference/plugin-source.html).

## Stucco pilot and validation

The first export uses the checked-in [Stucco 0.2.1 bundle](../plugins/stucco/README.md), with static-page and Axum recipes. Its analysis was imported, and its two recipes have recorded compilation checks. The upstream MIT and Apache-2.0 license texts are frozen evidence and survive export and skill installation. Retain those notices when distributing the entry.

The Rust integration tests check export determinism, relocation, reference links, hashes, rejection of incomplete verification, and preservation of existing files. To exercise the real Symposium 0.5.1 installer as well:

```sh
cargo build --locked --bin symposiast
cargo install symposium --version 0.5.1 --locked
# Populate Cargo's cache for the real pinned consumer project:
cargo fetch --locked --manifest-path plugins/stucco/bundle/verification/static-page/Cargo.toml
python3 scripts/check_registry.py
```

Pass `--cargo-agents /path/to/cargo-agents` for a separately installed CLI. The script creates an isolated `SYMPOSIUM_HOME`, relocates the exported registry, deletes its input bundle, and syncs the real pinned consumer into Claude Code and Codex skill directories. It verifies all installed bytes, repeats sync, and checks removal after changing the dependency to a path-crate fixture with a mismatched version. The fixture tests selection only, not Stucco runtime behavior. No user-wide agent configuration is changed.

## Submit guidance or request coverage

Read the registry’s [contribution guide](https://github.com/mcaveniathor/symposiast-registry/blob/main/CONTRIBUTING.md) for the full workflow and review criteria.

- **Submit an entry:** fork the registry, generate guidance for an exact release, inspect its citations and license notices, verify every recipe, and export into a new entry directory. Add its summary and tags to `catalog.json`, run registry and site checks, and open a pull request with the full bundle or reproducible generation details. Keep separate crate releases in separate directories.
- **Request a crate or release:** search [existing issues](https://github.com/mcaveniathor/symposiast-registry/issues), then use the [guidance request form](https://github.com/mcaveniathor/symposiast-registry/issues/new?template=guidance-request.yml). Include the package, release, intended tasks, features, and target constraints. No generated bundle or agent credentials are needed. Requests do not guarantee a delivery date.
- **Report a correction:** use the [correction form](https://github.com/mcaveniathor/symposiast-registry/issues/new?template=correction.yml) with the entry, finding or recipe ID, and a reproduction. Regenerate affected records when submitting a fix; do not manually alter provenance to bypass validation.

Maintainers review source identity, citations, scope, licenses, recipe usefulness, and verification records before accepting an entry. Compilation and recorded agent review are shown separately. An absent review is not approval; CI does not establish recommendation quality or runtime behavior.

## Publication and the registry website

The pilot is published in a separate Git registry with a catalog README and a [revisioned bundle download](https://github.com/mcaveniathor/symposiast-registry/releases/tag/stucco-0.2.1-r1). The archive preserves the full source bundle, runnable recipe projects, compiler logs, and recorded lockfiles; a SHA256 file accompanies it. Corrections should get new release tags and archives rather than replacing an existing download.

The [registry website](https://mcaveniathor.github.io/symposiast-registry/) is built with Stucco in the registry repository and deployed through GitHub Pages. Its searchable catalog reads `catalog.json` and each entry’s exported provenance; the contribution page renders the repository’s `CONTRIBUTING.md`. Accepted entries appear after the Pages build deploys. See [site development](https://github.com/mcaveniathor/symposiast-registry/blob/main/site/README.md) for local preview and checks.

Automated registry PRs and broader crate coverage remain follow-up work. Bevy is not included until its guidance and recipes pass review and compilation.
