# Documentation site

The site is a static Rust program built with [Stucco](https://github.com/mcaveniathor/stucco), pinned to version 0.2.1. It uses Stucco's page renderer, components, theme bundle, and content-hashed assets. It needs no server at runtime.

From the repository root:

```sh
cargo run --locked -p symposiast-site -- dist --base /symposiast/
python3 site/check_links.py dist --base /symposiast/
```

`--base` controls links and asset paths. Use `/` for a site hosted at the domain root. `dist/` is ignored by Git. Edit the Rust page shell in `site/src/`, prose in `site/content/`, and styles in `site/assets/`. The `site/downloads/` tree contains the eight validated, standalone Cargo projects copied into the site's `examples/` directory.

The [automation guide](content/automation.md) documents the reusable Symposiast action and release workflow examples.

For a local preview at the root path:

```sh
cargo run --locked -p symposiast-site -- dist --base /
python3 -m http.server 8000 --directory dist
```

## GitHub Pages

The Documentation workflow builds and checks the site for pull requests and branch pushes. It uploads a Pages artifact. Only `main` deploys; unmerged changes do not replace the public site.

In the repository's **Settings → Pages**, select **GitHub Actions** as the build source. Merge the documentation changes into `main` to trigger the first deployment. The resulting project site is `https://mcaveniathor.github.io/symposiast/`. The workflow can also be run manually once it exists on the default branch.

The workflow requires Pages to be enabled for the repository and the `github-pages` environment to permit deployments from `main`. No custom domain or third-party hosting credentials are required.
