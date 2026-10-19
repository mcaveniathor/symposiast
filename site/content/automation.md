# Automation

Generate release-specific crate guidance in GitHub Actions with `mcaveniathor/symposiast@action-v1.0.0`. The action collects the published crate and pinned repository evidence, runs your chosen agent, validates citations, compiles complete consumer recipes, and uploads a portable bundle for installation through Symposium.

Use it after `cargo publish`, when a GitHub release is published, or from a manual workflow. Consumers choose when to install the resulting guidance; generation does not change their dependencies or installed plugins.

## Run on a release

Copy the [complete release workflow](https://github.com/mcaveniathor/symposiast/blob/action-v1.0.0/examples/workflows/release-guidance.yml) into `.github/workflows/release-guidance.yml` in your crate repository. Change its crate name, add the selected provider secret, and commit it to the default branch. It supports published releases and manual dispatch. Generation uses read-only repository permissions; a separate job attaches the archive to the GitHub release.

The publication step refuses to overwrite an existing release asset. Remove or rename an older asset deliberately before rerunning publication. Manual dispatch creates a workflow artifact without attaching it to a release.

The central step is:

```yaml
- name: Generate crate guidance
  id: guidance
  uses: mcaveniathor/symposiast@action-v1.0.0
  with:
    crate: your-crate
    version: ${{ github.event.release.tag_name }}
    agent: claude-code
  env:
    ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```

Install and authenticate the agent before this step, as shown below. The action installs Symposiast itself unless you supply `symposiast-executable`.

The version must be an exact semantic version, optionally prefixed with `v`: `1.2.3`, `v1.2.3`, and `1.2.3-rc.1` are accepted. For monorepo tags such as `my-crate-v1.2.3`, map the tag to the crate's exact version in an earlier step. Put event values in environment variables before processing them in a shell; do not insert tag text directly into shell commands.

GitHub release publication and crates.io publication are separate operations. Publish the crate before generating its guidance. With an exact version, the action polls crates.io for up to `registry-wait-seconds` before starting analysis. This covers registry propagation; it does not retry paid agent calls. Omitting `version` resolves the latest stable, non-yanked version at run time, so specify it for release automation.

If your existing publish workflow runs `cargo publish`, place the action after that succeeds, or in a dependent job using `needs`. Use a workflow `timeout-minutes` large enough to cover CLI compilation, registry propagation, analysis, and every recipe check. The action's `timeout` applies separately to each agent invocation and each recipe check.

## Prepare the runner and agent

Use an ephemeral Linux runner such as `ubuntu-latest`, with Python 3.11 or newer and rustup available. The action installs the Rust toolchain pinned in its source and builds its own CLI with `cargo install --locked`. If you supply `symposiast-executable`, it skips both installation steps: make that executable and the selected Rust toolchain available beforehand. It does not install, log in to, or select a billing account for an analysis agent. Network access is required for the crate registry, source repositories, Cargo dependencies, and your provider.

When checking out a repository, avoid retaining its token for analysis and verification:

```yaml
permissions:
  contents: read

# Inside jobs.<job>.steps:
- uses: actions/checkout@v4
  with:
    persist-credentials: false
- uses: actions/setup-node@v4
  with:
    node-version: '22'
```

Pin provider CLI versions so an unrelated CLI upgrade does not change the integration unexpectedly. These versions match the documented adapter interfaces; they do not establish that a particular account has working authentication or provider access.

### Claude Code

Install Claude Code in a preceding step, then expose its key only to the generation step:

```yaml
- name: Install Claude Code
  run: npm install --global @anthropic-ai/claude-code@2.1.303
- name: Generate guidance
  uses: mcaveniathor/symposiast@action-v1.0.0
  with:
    crate: clap
    version: '4.6.7'
    agent: claude-code
  env:
    ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```

Add `ANTHROPIC_API_KEY` as a repository or environment secret. Use the provider's documented configuration for another supported authentication arrangement.

### Codex

Codex is the default agent. Install its CLI, authenticate without putting the key in an argument, and run the action:

```yaml
- name: Install Codex
  run: npm install --global @openai/codex@0.159.2
- name: Authenticate Codex
  env:
    OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
  run: |
    test -n "$OPENAI_API_KEY"
    printenv OPENAI_API_KEY | codex login --with-api-key
- name: Generate guidance
  uses: mcaveniathor/symposiast@action-v1.0.0
  with:
    crate: axum
    version: '0.8.9'
    agent: codex
```

The login writes provider state on the disposable runner. The state directory must remain writable. Do not enable shell tracing for authentication steps or upload provider state as an artifact.

### Custom agents, review, and imported analysis

Set `agent: custom` and `agent-executable` to a trusted executable implementing the [adapter contract](adapters.html#custom-executable-contract). Install it before the action runs. Custom executables receive a structured job on standard input and return an Analysis object on standard output.

Use `reviewer` and, optionally, `reviewer-executable` to add an independent review. Install and authenticate that provider too. A rejected review stops generation; a reviewer is not an automatic fallback after an analysis failure.

For a reviewed analysis file, set `analysis-file: path/to/analysis.json`. The action imports it instead of calling an analysis agent, while citation validation, optional review, and recipe compilation still run. Its source IDs and line ranges must match the exact evidence collected for this run. An analysis from a different release or collection is not automatically reusable. Import mode does not need an analysis provider's credentials.

## Action inputs

| Input | Default | Meaning |
| --- | --- | --- |
| `crate` | Required | Public crates.io package name. |
| `version` | Latest stable | Exact version, with an optional leading `v`. Prereleases require an explicit version. |
| `agent` | `codex` | `codex`, `claude-code`, or `custom`. Unused for analysis when `analysis-file` is supplied. |
| `agent-executable` | Adapter default | Installed analysis executable or wrapper. Required for a custom agent. |
| `reviewer` | None | Optional `codex`, `claude-code`, or `custom` reviewer. |
| `reviewer-executable` | Adapter default | Installed reviewer executable or wrapper. Required for a custom reviewer. |
| `analysis-file` | None | Structured Analysis JSON to import for this collection. |
| `timeout` | `600` | Positive seconds per agent invocation and per recipe check. |
| `registry-wait-seconds` | `300` | How long to wait for an explicitly requested version on crates.io before analysis. `0` skips polling. |
| `rust-toolchain` | Action's toolchain pin | Rust toolchain for building Symposiast and checking recipes. |
| `symposiast-executable` | Build from action source | Optional preinstalled CLI executable. Skips Rust and CLI installation; you control their availability and compatibility. |
| `output-directory` | Fresh directory in `RUNNER_TEMP` | Bundle output; must be new or empty. |
| `artifact-name` | `symposiast-<crate>-<version>` | Name of the uploaded workflow artifact, using the resolved version. |
| `upload-artifact` | `true` | Whether to upload the generated archive as a workflow artifact. |
| `retention-days` | `14` | Workflow artifact retention, subject to repository settings. |

The source used to build Symposiast comes from the action reference, independently of the consumer checkout. `action-v1.0.0` names a fixed action release; `action-v1` tracks compatible updates. Pin the full commit SHA when your workflow requires immutable dependency references. Overriding the toolchain or executable changes the tested combination and is recorded in the generated verification results where applicable.

## Outputs and installation

| Output | Meaning |
| --- | --- |
| `bundle-path` | Absolute path to the generated bundle on this runner. |
| `archive-path` | Absolute path to the portable compressed archive on this runner. |
| `crate-version` | Exact resolved crate version. |
| `artifact-name` | Name selected for the workflow artifact. |
| `artifact-id` | GitHub artifact ID when uploading is enabled. |
| `artifact-url` | GitHub artifact download URL when uploading is enabled. |

Use `${{ steps.guidance.outputs.archive-path }}` in a later step in the same job. Paths are local to that runner; use upload/download artifact steps to transfer the archive to another job. Downloading a workflow artifact requires GitHub access, and workflow artifacts expire according to their retention settings. Attaching the archive to a release provides a separate distribution path; see the [release workflow](https://github.com/mcaveniathor/symposiast/blob/action-v1.0.0/examples/workflows/release-guidance.yml).

The archive includes `bundle.json`, frozen `sources/`, and verified consumer projects, lockfiles, and compiler logs from `verification/`. It excludes the generated `plugin/` directory, build caches, and runtime configuration tied to the runner. Inspect coverage warnings, citations, and compilation results before installing.

The archive is named `<crate>-<version>.tar.gz` and contains a `<crate>-<version>/` directory. Download the workflow artifact, unzip its wrapper if needed, then extract and install the bundle locally. For the clap example:

```sh
tar -xzf clap-4.6.7.tar.gz
symposiast install clap-4.6.7
cd /path/to/consumer
cargo agents sync
```

Use a durable local Symposiast installation and have Symposium configured. Installation renders the skills, MCP registration, and version hook with paths that belong to the recipient's machine. The plugin targets the exact analyzed crate version; the hook can report dependency drift. See [installation details](getting-started.html#install-into-your-project).

## Permissions, failures, and limits

Keep generation in a job with `contents: read`. Give `contents: write` only to a separate publication job that downloads the successful archive and attaches it to the intended release. Avoid exposing provider secrets to untrusted pull requests, including `pull_request_target` workflows that check out contributor code. Review custom executables and imported analysis just as you would other executable workflow inputs.

Compilation can run dependency build scripts and procedural macros. The verifier bounds time and inherited environment variables but does not create an operating-system sandbox. Use disposable runners for trusted crates, and a suitably isolated environment when validating untrusted dependencies. Compilation results mean `cargo check` passed for the recorded recipe and lockfile; they do not establish runtime behavior.

Collection, analysis, citation validation, review, or compilation failures stop the action. It uploads the portable archive only after successful generation. Registry polling happens before analysis; the action does not retry an agent invocation automatically. Fix authentication or input problems, then deliberately rerun the job. A rerun can resolve newer evidence or dependencies when versions are not pinned.

The action's automated checks exercise subprocess and artifact contracts with fixtures. They do not claim a successful hosted run with your provider credentials. The existing [validated crate examples](examples.html) use imported analyses with compiled consumers; a live provider run additionally depends on the selected CLI, authentication, account access, and network policy.
