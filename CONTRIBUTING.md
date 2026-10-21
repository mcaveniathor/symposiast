# Contributing to Symposiast

Help make release-specific Rust guidance useful in real projects. Start with a [maintainer pilot](https://mcaveniathor.github.io/symposiast/maintainers.html), [try existing guidance](https://mcaveniathor.github.io/symposiast-registry/subscribe.html), or [report first or repeat use](https://github.com/mcaveniathor/symposiast-registry/issues/new?template=usage-feedback.yml).

## Choose the right repository

- Generator, action, or documentation changes belong in this repository. Include a reproduction for bugs and describe the user-facing outcome for changes.
- Guidance entries, coverage requests, corrections, and consumer usage reports belong in the [community registry](https://github.com/mcaveniathor/symposiast-registry/blob/main/CONTRIBUTING.md).
- Symposium's [official recommendations](https://github.com/symposium-dev/recommendations) have their own review process. Read their current contribution rules before proposing an upstream change. This registry is independent of that collection.

Keep credentials, agent login state, private code, and unredacted conversations out of issues and pull requests.

## Review and validation

Keep changes focused on a concrete user need. Preserve exact-release matching, source evidence, license notices, and honest verification status. Distinguish compilation, runtime checks, agent review, and human review. Do not change recorded provenance to bypass checks or imply endorsement without the maintainer's agreement.

For Rust changes, run:

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
```

For site changes, also follow [site validation](site/README.md), including root and project URL bases and desktop and narrow layouts. For action changes, run `python3 -m unittest discover -s .github/action -p 'test_*.py' -v` and lint changed workflows with actionlint. Provider-backed runs are separate from fixture checks; document which you actually performed.

## Maintainer follow-through

For a pilot, record the chosen tasks, exact release, reviewer, consumer feedback, correction owner, and next-release decision in the issue. Review a successful candidate artifact before distributing it. A second use or release is a stronger adoption signal than a download count. Use the [maintainer guide](https://mcaveniathor.github.io/symposiast/maintainers.html) for the complete workflow.
