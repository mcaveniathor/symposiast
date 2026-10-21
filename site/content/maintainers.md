# Guidance for your crate

Help developers use your crate with their coding agents. Start with one published release and a few real tasks, review the resulting guidance, then try it with users before adopting a recurring release workflow.

Symposiast produces the guidance and its evidence. [Symposium](https://symposium.dev/) distributes it to configured agents. Developers can use a portable registry entry without installing Symposiast or authenticating an analysis agent.

## Start a maintainer pilot

[Propose a pilot](https://github.com/mcaveniathor/symposiast/issues/new?template=maintainer-pilot.yml) with your crate, exact published release, and up to three tasks that users regularly struggle with. Include the relevant features and target, a link to existing examples, and who can review the advice. Contributors can also propose a pilot; say whether a crate maintainer is involved.

The first milestone is one reviewed bundle used on a real task. A proposal starts a public scoping conversation; it does not guarantee delivery, ongoing coverage, or endorsement by the crate's maintainers. Keep credentials and private code out of the issue.

Agree on these details before generating:

- The exact crates.io release, features, target, and supported use cases.
- What a successful solution to each task should do, including an executable check where practical.
- Who will review the findings and recipes, and who can try them in a consumer project.
- Who will maintain corrections and decide whether to repeat the process for another release.

## Produce a reviewable bundle

Follow [getting started](getting-started.html) to install the CLI and select an authenticated agent. Use an explicit version and a new output directory:

```sh
symposiast build your-crate --version 1.2.3 --agent codex \
  --output .symposiast/your-crate-1.2.3
```

The generator samples release evidence; it does not currently accept a task brief as a CLI input. Compare the output with the agreed tasks. If recipes need changes, prepare an Analysis JSON file using the bundle's existing source IDs and citations, then import it with `symposiast analyze BUNDLE --analysis analysis.json`. The [adapter contract](adapters.html#custom-executable-contract) describes the structured input. Importing validates citations and recompiles recipes; it does not establish an agent review or maintainer approval.

Review the findings, citations, omitted evidence, features, recipe code, and recorded lockfiles. Run behavior checks for the agreed tasks separately and record the commands, results, toolchain, and platform in the pilot issue. **Compiled** means the recorded Cargo check passed; it does not mean the task works at runtime.

Record human review in the issue or pull request, including the reviewed bundle revision. Human review and the bundle's optional agent review are separate records. Preserve upstream notices and keep attribution factual. A link to Symposiast belongs in the release notes or provenance; the installed guidance should stay focused on the user's task.

## Choose a publishing path

| Path | When to use it | Review and distribution |
| --- | --- | --- |
| Symposiast community registry | Share a verified bundle for an already-published release. | Export, submit a PR, and use the registry's review process. Users subscribe explicitly. |
| Symposium official recommendations | Propose externally authored guidance for the default collection. | Submit the skill content for Symposium's own vetting under its contribution rules. Community registry acceptance is separate. |
| Skills shipped by the crate | A maintainer wants to own and distribute guidance with the crate. | Follow Symposium's crate-author guide, including its recommendations entry, and validate the packaged crate and consumer installation. |

For the community registry:

```sh
symposiast export-registry .symposiast/your-crate-1.2.3 \
  --output /path/to/registry/your-crate-1-2-3
```

The output path must not exist. Preserve the generated manifests, skill, evidence, provenance, and lockfiles together, then follow the [registry contribution guide](https://mcaveniathor.github.io/symposiast-registry/contributing.html). Export validates recorded evidence and compilation; it does not publish anything.

Symposium's [crate-author guidance](https://symposium.dev/crate-authors/supporting-your-crate.html) prefers skills shipped in the crate and requires externally authored skills to be submitted directly to its [recommendations repository](https://github.com/symposium-dev/recommendations) for vetting. Check its current contribution and AI policies before preparing an upstream submission. No Symposiast entry is automatically accepted or endorsed there.

Symposiast currently collects **published** crates.io releases. A post-release run cannot insert files into that release's immutable crate archive. Use a registry entry or release attachment for that bundle. Embedding guidance into a future crate release needs a separate packaging and validation process; do not relabel an older bundle or carry its exact-version predicate into a different release. The existing action is a post-release workflow, not a prepublication crate-skills generator.

## Try it with three developers

Invite willing users to follow the [consumer installation guide](https://mcaveniathor.github.io/symposiast-registry/subscribe.html) on a project that actually uses the supported release. Do not change their dependency versions just to make the guidance match.

For each trial, ask them to report:

1. Whether the correct release's skill installed and appeared in `cargo agents status`.
2. Whether the agent consulted a finding or recipe while completing the task.
3. Which project checks passed, where they got stuck, and whether the guidance helped.
4. Whether they used it again on a different task or session.

[Share a usage report](https://github.com/mcaveniathor/symposiast-registry/issues/new?template=usage-feedback.yml). First use, unsuccessful attempts, and repeat use are all useful. Reports are voluntary and public; a short redacted description is enough. No prompt transcript or private repository is required. Installation and downloads alone do not show that guidance was used, and a successful task alone does not prove it improved agent performance.

## Repeat for the next release

Use the [release workflow](automation.html) to generate a candidate bundle after publishing. It uploads an artifact for review. Inspect that exact artifact before publishing it; generating again can produce different guidance and requires a fresh review.

Keep one entry per exact crate release. Corrections get a new bundle revision and preserve old downloads. Track generation cost and review time, reuse existing evidence when appropriate, and choose releases based on user demand. The action bounds invocation time but does not set a provider spending cap; configure that through your provider if needed.

A useful pilot finishes with one external maintainer involved, three real consumer trials, and one repeat use or release. Record the actual outcomes in the pilot issue before expanding coverage. These are learning milestones, not claims of current adoption.

## Contribute to the project

Help improve Symposiast and its registry by trying guidance, reviewing recipes, reporting corrections, or maintaining a crate's coverage. Share a link to a useful release with developers who need it.

[Propose a pilot](https://github.com/mcaveniathor/symposiast/issues/new?template=maintainer-pilot.yml), [report your experience](https://github.com/mcaveniathor/symposiast-registry/issues/new?template=usage-feedback.yml), or [contribute verified guidance](https://mcaveniathor.github.io/symposiast-registry/contributing.html).
