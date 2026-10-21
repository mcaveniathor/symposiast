# Validation

The initial implementation was checked with Rust 1.99.1 and Symposium 0.5.1.

## Live generated clap, axum, and Bevy recipes (October 21, 2026)

The current documentation bundles were produced by `symposiast analyze --agent codex --timeout 900` using authenticated Codex CLI 0.163.0 and generator revision `19cd230`. The live responses supplied both findings and complete recipe code, including example coverage; no hand-written analysis was imported. All recipe code is retained unchanged from the generator. No separate reviewer was invoked. This successful run supersedes the earlier live-provider limitations described below for Codex; it does not establish a live Claude Code run.

| Crate | Release | Findings | Recipes compiled | Documentation examples | Adapted / supporting / skipped |
| --- | --- | --- | --- | --- | --- |
| clap | 4.6.7 | 13 | 8 | 122 | 14 / 6 / 102 |
| axum | 0.8.9 | 14 | 13 | 78 | 30 / 27 / 21 |
| Bevy | 0.20.0 | 7 | 5 | 5 | 3 / 0 / 2 |

Clap and axum retain the earlier exact release targets. Bevy 0.20.0 was resolved as the latest stable, non-yanked release from the crates.io index. Collection verified each archive checksum. The Bevy repository archive exceeded the download limit, leaving published-crate evidence only, with two truncated manifests and 530 eligible files omitted. This is sampled guidance, not exhaustive engine coverage. All bundles retain their collection and extraction warnings. Upstream license files were copied from checksum-verified crate archives and separately hashed into the bundles.

All 26 generated recipes passed `cargo check --locked --all-targets` with Rust 1.99.1. The Bevy window, mesh, and profiling checks initially failed because Wayland development files were missing. Reverification passed with Ubuntu Wayland and xkbcommon packages extracted under `/tmp`, supplied through a Cargo wrapper that sets `PKG_CONFIG_PATH`; the generated code and features were unchanged. CI installs the corresponding development packages normally. Compilation does not establish graphical execution.

Separate runtime checks passed:

- All eight clap executables ran their generated assertions. `cargo test --locked` also passed, including the two generated parser-definition tests.
- Twelve axum servers passed 27 actual localhost HTTP scenarios for JSON, routes and fallbacks, shared state, path/query extraction, rejections, custom extractors, demonstration authentication middleware, service failures and timeouts, response composition, forms, URI rewriting, and tracing/layer composition. The thirteenth executable checked bounded body collection and its error. `python3 scripts/check_axum_examples.py` repeats these checks and stops only the processes it starts. Port 3000 must be free.
- Bevy's `hello` and `message` executables passed. The window, mesh, and profiling recipes were not run or visually checked.

`plugins/{clap,axum,bevy}/bundle` preserves the actual analysis, evidence, lockfiles, compilation logs, and generation provenance. Temporary compiler-log path prefixes are normalized; recipe fingerprints and lockfile hashes are unchanged. Original agent summaries describe their pre-verification state; compilation records are the authority for the subsequent checks. No user-wide plugin installation or synchronization was performed.

Final local validation passed formatting, Clippy with warnings denied, all 55 Rust tests, actionlint, both documentation generators’ drift checks, and site builds/link checks at `/symposiast/` and `/` (eight HTML pages and 394 local references each). Browser inspection at the default desktop viewport confirmed the overview and Bevy section, preserved Rust generic types, no horizontal document overflow, and no console errors. The expanded recipe navigation now scrolls within the viewport. Mobile layout was not separately rechecked.

The site now contains 32 consumer projects, including the six existing Stucco examples. `python3 scripts/update_examples.py --check` checks generated clap, axum, and Bevy prose, downloads, and coverage reports; the existing Stucco check remains in place. Bundle rendering validates source checksums, complete coverage associations, recipe fingerprints, and lockfiles. CI validates the bundles, checks generated documentation, compiles all new recipes, and runs the recorded clap, axum, and headless Bevy checks.

## Automated checks

- Formatting passes `cargo fmt --all --check`.
- Linting passes `cargo clippy --locked --workspace --all-targets -- -D warnings`.
- All 39 tests pass with `cargo test --locked --workspace --all-targets`.

Tests cover archive checksums and traversal, release selection, bounded evidence selection, real fixture subprocesses for agent adapters, structured responses, timeout cleanup, source citations, stale verification results, transactional replacement, plugin installation, MCP stdio requests, and version/provenance warnings.

## Initial public-crate integration (historical)

The collector resolved clap 4.6.7 and axum 0.8.9 from the crates.io index, verified their published archives, and fetched their pinned repository snapshots. Each bundle contains 48 evidence sources and records coverage limits.

Three recipes per crate were supplied through the structured analysis import path. All six compiled against their exact released crate versions with recorded dependency lockfiles. Additional behavior checks passed:

- clap: six parser tests across derive, subcommand, and fallible-validation recipes.
- axum: three in-memory request examples exercised routing, shared state, and typed JSON, with response assertions and no listening sockets.

These behavior checks are separate from the CLI's recipe verification status, which intentionally records compilation only. Pilot bundles and logs are retained under the ignored `.symposiast/` directory in the development workspace.

The exporter supports both `SYMPOSIUM.toml`, recognized by Symposium 0.5.1, and `Symposium.toml`, used in the current upstream documentation. Both files contain the same manifest.

The official Symposium 0.5.1 recursive validator recognizes the real pilot manifests and skills. In an isolated Symposium configuration, sync installed the clap skill and MCP registration; the registered MCP command returned the compiled recipe. Native SessionStart dispatch for both Claude Code and Codex produced no mismatch warning for the matching release and warned after the consumer resolved clap 4.6.6.

An upstream limitation remains: Symposium 0.5.1 removed the mismatched skill during sync but retained the existing MCP registration. The hook and release metadata identify that mismatch; automatic removal of the stale MCP entry is not established.

## Initial live agent limitations (historical)

Built-in command interfaces were checked against Claude Code 2.1.303 and Codex 0.159.2, and their subprocess contracts are covered by fixture tests. Live provider responses were not verified in this environment:

- Claude Code reports no active login.
- Codex reports an existing login, but its injected state directory is read-only. Startup fails before a provider request because required state files cannot be written.

No credentials were copied or replaced. A normal authenticated CLI with writable runtime state is needed to validate the live analysis path. The successful public-crate integration checks used imported analyses and do not establish live model integration.

## Stucco 0.3.0 plugin and examples

The current checked-in Stucco guidance targets 0.3.0 at repository commit `07335e73dfb22aba2ce89487bcee634a24c2080e`. A fresh exact-release collection verified the published archive checksum and matched all 31 sources of the existing 0.3.0 analysis. Two separately hashed upstream license texts bring the portable bundle to 33 frozen sources. The prior bundle and downloads have been replaced.

The refined analysis was imported through Symposiast and has 12 findings, six complete consumer recipes, and explicit coverage for all 57 extracted documentation examples: 14 adapted, 13 supporting, and 30 skipped with reasons. No separate reviewer was invoked for this revision; the bundle's `review` is null. Source selection and extraction remain bounded, and coverage decisions do not prove semantic completeness.

All six recipes passed `cargo check --locked --all-targets` against pinned lockfiles and their executable assertions passed separately:

- `static-page`: escaped text, main landmark, skip link, and exact exported asset bytes.
- `axum-fragments`: full pages, named fragments, unknown-target fallback, escaping, response headers, and stylesheet bytes through in-memory requests. The optional listening-server path was not used for these checks.
- `derived-table`: escaped values and stored-cents display formatting.
- `bound-validation`: invalid submitted state, validation messages, and invalid-field markup.
- `inline-server-theme`: a server-selected dark scheme, inline CSS, and document content.
- `status-details`: native progress, meter, details/summary, and status/detail content.

Compiler logs normalize temporary build-directory prefixes. Input fingerprints and lockfile hashes remain intact. `scripts/update_stucco_examples.py --check` verifies that all 18 downloadable Stucco files and the generated Stucco guide section match the checked-in bundle. CI runs every Stucco example executable and rejects stale downloads or prose.

## Documentation and workflow skill

The earlier Stucco migration pinned the documentation site to Stucco 0.3.0 and included twelve downloadable clap, axum, and Stucco projects. Site builds and link checks cover both `/symposiast/` and `/` deployment bases. No browser was connected for this migration, so desktop/mobile layout and browser enhancement were not visually revalidated. Earlier browser checks described below apply only to their earlier revisions.

## Release guidance action

The action suite passes 21 local tests covering input handling, exact-version polling, failure propagation, portable artifact integrity, and the deterministic Stucco fixture adapter. The Rust regression suite additionally invokes that adapter against real frozen evidence with and without the optional repository sources. All 55 Rust tests pass, along with formatting and Clippy with warnings denied. Actionlint validates the repository workflows and the release workflow example.

The CI fixture uses a custom executable adapter to return fixed guidance with coverage IDs from the actual collection. Its release identity and cited source hashes must match the checked-in published evidence. The recipe adapts the published README example; other inventoried examples receive explicit scope-based skips. This exercises the version 2 protocol without provider credentials or a model call. The fixture JSON by itself is not a complete import for an inventoried bundle.

The prebuilt-CLI action path collected Stucco 0.3.0, compiled the consumer, and packaged a portable archive. The recipient check validates the archived coverage and source hashes, runs the recipe's HTML assertions, and installs the plugin in a disposable directory. This does not enable guidance in the user's workspace. Hosted artifact upload and provider authentication are separate from these local checks.

## Portable registry export

The registry tests validate the current bundle's deterministic export, unchanged source bundle, relocation, relative references, evidence and lockfile hashes, coverage preservation, invalid/stale compilation rejection, and existing-path protection. Both upstream licenses accompany the exported evidence. The registry integration workflow targets the current version and uses a synthetic 0.3.1 path crate solely to test removal after a version mismatch.

The public registry already lists [Stucco 0.3.0](https://github.com/mcaveniathor/symposiast-registry/tree/main/stucco-0-3-0) and its [revisioned release archive](https://github.com/mcaveniathor/symposiast-registry/releases/tag/stucco-0.3.0-r1). This repository's refinements add example coverage and runtime assertions; they are not claimed to be present in that independently published archive. No user-wide plugin enablement or synchronization was performed during this migration.

## Maintainer and consumer adoption workflow

The maintainer guide, consumer trial instructions, and pilot/usage issue forms distinguish generation, human review, installation, observed agent consultation, task outcome, and repeat use. The release workflow example now uploads a candidate for review without an automatic publication job. Its generator remains pinned to `action-v1.0.0`; the revised example lives in the current source tree rather than that immutable tag.

Local validation for this change passed workspace formatting, Clippy with warnings denied, all 44 Rust tests, and the 16 action helper tests. Actionlint 1.7.7 passed the release example and repository workflows. Issue-form YAML, unique field IDs, dropdown options, and embedded workflow shell syntax were checked.

The generator site has eight HTML pages and the registry site has six. Both built and passed local link/asset checks at their project prefixes and at `/`. Chromium checked nine affected pages at widths 1440, 1024, 800, 390, and 320 with no document overflow or page-script errors. WCAG A/AA checks passed for those nine pages at width 390. Desktop and mobile screenshots were inspected. Registry search, the release-to-task navigation, and installation navigation without JavaScript passed. The registry's provenance/catalog validation and eight checker tests also passed.

These are local documentation and workflow checks. They do not establish a hosted provider run, public deployment, acceptance by Symposium's recommendations maintainers, or external user adoption. No generated skills or recorded verification evidence were changed.
