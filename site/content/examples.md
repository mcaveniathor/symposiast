# Examples you can inspect and run

These eight recipes were checked with Rust 1.99.1. Each project pins its primary crate and includes the lockfile used for validation. The versions below are **validated snapshots**, not a promise that they are still the latest release.

| Crate | Version | Recipes | Additional behavior checks |
| --- | --- | --- | --- |
| clap | 4.6.7 | Derive arguments, subcommands, fallible validation | Six parser tests passed |
| axum | 0.8.9 | Typed routes, shared state, JSON bodies | Three in-memory requests passed |
| Stucco | 0.2.1 | Static pages and Axum documents | HTML, escaping, asset bytes, and HTTP response assertions passed |

The pilot used imported structured analyses. Live Claude Code and Codex responses were not verified in the development environment. The normal `symposiast build` workflow uses your configured agent; see [agent adapters](adapters.html).

## Generate your own plugin

Omit `--version` to resolve the latest stable, non-yanked release at collection time. To target the exact releases shown here:

```sh
symposiast build clap --version 4.6.7 --agent codex \
  --output .symposiast/clap-4.6.7
symposiast build axum --version 0.8.9 --agent claude-code \
  --output .symposiast/axum-0.8.9
```

These commands collect evidence, validate the analysis, compile the recipes, and render a plugin. Recipe verification means `cargo check --locked --all-targets` passed; the behavior checks described on this page were run separately.

## Run the published recipes

Clone the [repository](https://github.com/mcaveniathor/symposiast), then use the checked-in projects directly. No analysis agent or Symposium installation is required to run them:

```sh
cargo test --locked --manifest-path site/downloads/clap/derive-cli/Cargo.toml
cargo run --locked --manifest-path site/downloads/clap/derive-cli/Cargo.toml -- --name Ada --count 2
cargo run --locked --manifest-path site/downloads/axum/typed-route/Cargo.toml
```

The axum examples send requests in memory through Tower; they do not open a listening socket. A successful run exits without output. Alternatively, save each linked `Cargo.toml`, `Cargo.lock`, and `src/main.rs` with that directory structure and run Cargo from the project directory.


## clap 4.6.7

Enable the `derive` feature to represent arguments with Rust types. Use `try_parse_from` in tests to inspect parse failures without terminating the process.


### Typed arguments with a default

A complete greeting CLI with a typed repetition count and parser-definition checks.

[Cargo.toml](examples/clap/derive-cli/Cargo.toml) · [Cargo.lock](examples/clap/derive-cli/Cargo.lock) · [src/main.rs](examples/clap/derive-cli/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-derive-cli"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{CommandFactory, Parser};

#[derive(Debug, Parser)]
#[command(version, about = "Print a greeting")]
struct Cli {
    #[arg(short, long)]
    name: String,
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let cli = Cli::parse();
    for _ in 0..cli.count {
        println!("Hello, {}!", cli.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_explicit_and_default_counts() {
        let cli = Cli::try_parse_from(["greet", "--name", "Ada", "--count", "2"]).unwrap();
        assert_eq!(cli.name, "Ada");
        assert_eq!(cli.count, 2);
        assert_eq!(Cli::try_parse_from(["greet", "-n", "Ada"]).unwrap().count, 1);
    }
}
```


```sh
cargo test --locked --manifest-path site/downloads/clap/derive-cli/Cargo.toml
```


Release evidence: [documentation, lines 86–146](https://docs.rs/crate/clap/4.6.7/source/src/_derive/_tutorial.rs) · [documentation, lines 239–245](https://docs.rs/crate/clap/4.6.7/source/src/_derive/_tutorial.rs) · [api, lines 70–78](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L70-L78).


### Required typed subcommands

A complete CLI that dispatches add and list subcommands through an enum.

[Cargo.toml](examples/clap/subcommand-cli/Cargo.toml) · [Cargo.lock](examples/clap/subcommand-cli/Cargo.lock) · [src/main.rs](examples/clap/subcommand-cli/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-subcommand-cli"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about = "Manage a small task list")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Add { name: String },
    List {
        #[arg(long)]
        all: bool,
    },
}

fn main() {
    match Cli::parse().command {
        Command::Add { name } => println!("Add {name}"),
        Command::List { all } => println!("List tasks (include completed: {all})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_each_variant() {
        let cli = Cli::try_parse_from(["tasks", "add", "write tests"]).unwrap();
        assert!(matches!(cli.command, Command::Add { name } if name == "write tests"));
        let cli = Cli::try_parse_from(["tasks", "list", "--all"]).unwrap();
        assert!(matches!(cli.command, Command::List { all: true }));
    }

    #[test]
    fn requires_a_subcommand() {
        assert!(Cli::try_parse_from(["tasks"]).is_err());
    }
}
```


```sh
cargo test --locked --manifest-path site/downloads/clap/subcommand-cli/Cargo.toml
```


Release evidence: [documentation, lines 148–164](https://docs.rs/crate/clap/4.6.7/source/src/_derive/_tutorial.rs) · [example, lines 1–25](https://docs.rs/crate/clap/4.6.7/source/examples/git-derive.rs) · [api, lines 70–78](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L70-L78).


### Typed validation with inspectable errors

Reject malformed and out-of-range ports, while keeping parser failures inspectable in tests.

[Cargo.toml](examples/clap/fallible-validation/Cargo.toml) · [Cargo.lock](examples/clap/fallible-validation/Cargo.lock) · [src/main.rs](examples/clap/fallible-validation/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-fallible-validation"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{error::ErrorKind, Parser};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
}

fn main() {
    match Cli::try_parse() {
        Ok(cli) => println!("Selected port {}", cli.port),
        Err(error) => error.exit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_valid_port() {
        assert_eq!(Cli::try_parse_from(["server", "--port", "8080"]).unwrap().port, 8080);
    }

    #[test]
    fn rejects_invalid_and_zero_ports() {
        for value in ["zero", "0", "65536"] {
            let error = Cli::try_parse_from(["server", "--port", value]).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::ValueValidation);
        }
    }
}
```


```sh
cargo test --locked --manifest-path site/downloads/clap/fallible-validation/Cargo.toml
```


Release evidence: [documentation, lines 166–207](https://docs.rs/crate/clap/4.6.7/source/src/_derive/_tutorial.rs) · [api, lines 45–78](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L45-L78).


## axum 0.8.9

Use brace captures such as `/users/{id}` for axum 0.8 routes. Share state explicitly with `State`, and place body-consuming extractors such as `Json` after extractors that only read request parts.


### Typed route captures without a network server

Construct a router using current path syntax and dispatch an in-memory request through Tower.

[Cargo.toml](examples/axum/typed-route/Cargo.toml) · [Cargo.lock](examples/axum/typed-route/Cargo.lock) · [src/main.rs](examples/axum/typed-route/src/main.rs)

```toml
[dependencies.axum]
default-features = true
features = []
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt",
]
version = "1.44"

[dependencies.tower]
default-features = true
features = ["util"]
version = "0.5.2"

[package]
edition = "2024"
name = "recipe-typed-route"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{
    body::{to_bytes, Body},
    extract::Path,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt;

async fn user(Path(id): Path<u64>) -> String {
    format!("user:{id}")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Router::new().route("/users/{id}", get(user));
    let request = Request::builder().uri("/users/42").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"user:42");
}
```


```sh
cargo run --locked --manifest-path site/downloads/axum/typed-route/Cargo.toml
```


Release evidence: [changelog, lines 121–122](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [api, lines 47–94](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [example, lines 15–33](https://github.com/tokio-rs/axum/blob/c59208c86fded335cd85e388030ad59347b0e5ae/examples/compression/src/tests.rs#L15-L33).


### Read shared state through State

Use Arc for inexpensive state cloning and exercise the stateful handler through an in-memory request.

[Cargo.toml](examples/axum/shared-state/Cargo.toml) · [Cargo.lock](examples/axum/shared-state/Cargo.lock) · [src/main.rs](examples/axum/shared-state/src/main.rs)

```toml
[dependencies.axum]
default-features = true
features = []
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt",
]
version = "1.44"

[dependencies.tower]
default-features = true
features = ["util"]
version = "0.5.2"

[package]
edition = "2024"
name = "recipe-shared-state"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use std::sync::Arc;
use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::ServiceExt;

struct AppState {
    greeting: String,
}

async fn greeting(State(state): State<Arc<AppState>>) -> String {
    state.greeting.clone()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = Arc::new(AppState { greeting: "Hello from shared state".into() });
    let app = Router::new().route("/", get(greeting)).with_state(state);
    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), b"Hello from shared state");
}
```


```sh
cargo run --locked --manifest-path site/downloads/axum/shared-state/Cargo.toml
```


Release evidence: [api, lines 158–189](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [example, lines 15–33](https://github.com/tokio-rs/axum/blob/c59208c86fded335cd85e388030ad59347b0e5ae/examples/compression/src/tests.rs#L15-L33).


### Combine state with a typed JSON body

Keep State before Json, accept a typed request body, and return a created JSON response without opening a socket.

[Cargo.toml](examples/axum/typed-json/Cargo.toml) · [Cargo.lock](examples/axum/typed-json/Cargo.lock) · [src/main.rs](examples/axum/typed-json/src/main.rs)

```toml
[dependencies.axum]
default-features = true
features = ["json"]
version = "=0.8.9"

[dependencies.serde]
default-features = true
features = ["derive"]
version = "1"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt",
]
version = "1.44"

[dependencies.tower]
default-features = true
features = ["util"]
version = "0.5.2"

[package]
edition = "2024"
name = "recipe-typed-json"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use std::sync::Arc;
use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

struct AppState { prefix: String }

#[derive(Deserialize)]
struct Input { name: String }

#[derive(Serialize)]
struct Output { greeting: String }

async fn greet(
    State(state): State<Arc<AppState>>,
    Json(input): Json<Input>,
) -> (StatusCode, Json<Output>) {
    (StatusCode::CREATED, Json(Output {
        greeting: format!("{}, {}", state.prefix, input.name),
    }))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let state = Arc::new(AppState { prefix: "Hello".into() });
    let app = Router::new().route("/greet", post(greet)).with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/greet")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"Ada"}"#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[CONTENT_TYPE], "application/json");
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(body.as_ref(), br#"{"greeting":"Hello, Ada"}"#);
}
```


```sh
cargo run --locked --manifest-path site/downloads/axum/typed-json/Cargo.toml
```


Release evidence: [documentation, lines 120–152](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [api, lines 12–56](https://docs.rs/crate/axum/0.8.9/source/src/json.rs) · [api, lines 158–189](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [example, lines 15–33](https://github.com/tokio-rs/axum/blob/c59208c86fded335cd85e388030ad59347b0e5ae/examples/compression/src/tests.rs#L15-L33).


## Stucco 0.2.1

This documentation site is itself built with [Stucco](https://github.com/mcaveniathor/stucco): Rust components render the pages, and a `Bundle` supplies the hashed theme assets. These smaller recipes show the same building blocks without the documentation site's shell.

### Install the Stucco plugin

A frozen, verified Stucco bundle is included in the repository. With Symposiast and Symposium configured, run from the checkout:

```sh
symposiast install plugins/stucco/bundle
cd /path/to/consumer
cargo agents sync
```

Installation creates a local plugin with cited skills, MCP retrieval, and a version-check hook using your own executable and bundle paths. It needs no analysis-provider call. Guidance and MCP registration match Stucco **0.2.1** exactly; the hook can warn about other resolved versions. Restart the consuming MCP session after installation. See the [plugin guide](https://github.com/mcaveniathor/symposiast/blob/main/plugins/stucco/README.md) for inspection and refresh commands.

The bundle contains 27 automatically selected evidence files, one supplemental `stucco-core` API file from the same release commit, and two upstream license files. The source archive checksum authenticates the published facade archive; supplemental files have their own recorded hashes and commit URLs. The checked consumer lockfiles also resolve the Stucco companion crates to 0.2.1.

### Export a static page with hashed assets

Write index.html and every Bundle asset to an optional output directory (default stucco-example). BASE_PATH is /demo/ for a project site; change it to / for root hosting. Assertions check escaping, the main landmark, the skip link and exact asset bytes on disk.

[Cargo.toml](examples/stucco/static-page/Cargo.toml) · [Cargo.lock](examples/stucco/static-page/Cargo.lock) · [src/main.rs](examples/stucco/static-page/src/main.rs)

```toml
[dependencies.stucco]
default-features = false
features = [
    "layout",
    "typography",
]
version = "=0.2.1"

[package]
edition = "2024"
name = "recipe-static-page"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use stucco::prelude::*;

// Match the site's URL mount point; use "/" for a site served at the root.
const BASE_PATH: &str = "/demo/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("stucco-example"));
    let bundle = Bundle::new(Preset::Slate).prefix(&format!("{BASE_PATH}_stucco/"));
    let user_text = "A <careful> welcome & a fresh start.";
    let html = Page::new(&bundle, "Built with Stucco")
        .main(
            Container::new().child(
                Stack::new()
                    .child(Heading::new(1, "Built with Stucco"))
                    .child(Text::new(user_text))
                    .child(Link::new(
                        "Stucco repository",
                        "https://github.com/mcaveniathor/stucco",
                    )),
            ),
        )
        .render();

    assert!(html.contains(r#"<main id="main">"#));
    assert!(html.contains(r##"href="#main""##));
    assert!(html.contains("A &lt;careful&gt; welcome &amp; a fresh start."));
    assert!(!html.contains("<careful>"));
    assert!(html.contains(bundle.stylesheet_url()));

    fs::create_dir_all(&output)?;
    let paths = bundle.paths();
    for url in &paths {
        // The deployment prefix is a URL, not a directory inside the output.
        let relative = url
            .strip_prefix(BASE_PATH)
            .expect("asset stays under site prefix");
        let relative = Path::new(relative);
        assert!(
            relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        );
        let destination = output.join(relative);
        fs::create_dir_all(destination.parent().expect("asset has a parent"))?;
        let asset = bundle.get(url).expect("listed asset exists");
        fs::write(&destination, &*asset.bytes)?;
        assert_eq!(fs::read(&destination)?, asset.bytes.as_ref());
    }
    fs::write(output.join("index.html"), &html)?;
    assert_eq!(fs::read_to_string(output.join("index.html"))?, html);
    assert!(!paths.is_empty());
    println!(
        "Wrote {} and {} assets; serve it at {BASE_PATH}",
        output.display(),
        paths.len()
    );
    Ok(())
}
```

```sh
cargo run --locked --manifest-path site/downloads/stucco/static-page/Cargo.toml -- /tmp/stucco-preview/demo
```

Serve `/tmp/stucco-preview` as the document root and visit the `/demo/` path. The URL prefix must match `BASE_PATH`; opening the HTML directly as a file will not resolve its absolute asset URLs. For root hosting, change `BASE_PATH` to `/` and regenerate.

Release evidence: [documentation, lines 36–56](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/docs/guide/getting-started.md#L36-L56) · [documentation, lines 49–58](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/docs/guide/components.md#L49-L58) · [documentation, lines 144–160](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/docs/guide/recipes.md#L144-L160) · [api, lines 103–124](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/crates/stucco-core/src/bundle.rs#L103-L124) · [api, lines 152–168](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/crates/stucco-core/src/bundle.rs#L152-L168).

### Serve a document and its stylesheet through Axum

Return a Document from a PageCx handler, install Stucco last on the router, and issue in-memory requests for the HTML and its content-hashed CSS. Assertions cover status, content type, escaped text, landmarks, immutable cache policy and exact stylesheet bytes.

[Cargo.toml](examples/stucco/axum-page/Cargo.toml) · [Cargo.lock](examples/stucco/axum-page/Cargo.lock) · [src/main.rs](examples/stucco/axum-page/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = []
version = "0.8.9"

[dependencies.stucco]
default-features = false
features = [
    "layout",
    "typography",
    "axum",
]
version = "=0.2.1"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt",
]
version = "1.53"

[dependencies.tower]
default-features = true
features = ["util"]
version = "0.5.3"

[package]
edition = "2024"
name = "recipe-axum-page"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{
        Request, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    routing::get,
};
use stucco::prelude::*;
use tower::ServiceExt;

async fn index(page: PageCx) -> Document {
    page.title("Hello, Stucco").main(
        Container::new().child(
            Stack::new()
                .child(Heading::new(1, "Hello, Stucco"))
                .child(Text::new("Rust <HTML> & a real stylesheet.")),
        ),
    )
}

fn request(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = Bundle::new(Preset::Slate);
    let stylesheet_url = bundle.stylesheet_url().to_owned();
    let stylesheet = bundle
        .get(&stylesheet_url)
        .expect("theme stylesheet exists");
    // Call stucco last so it provides assets, middleware and PageCx to every route.
    let app = Router::new().route("/", get(index)).stucco(bundle);

    let response = app.clone().oneshot(request("/")).await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/html; charset=utf-8");
    let html = String::from_utf8(to_bytes(response.into_body(), 1024 * 1024).await?.to_vec())?;
    assert!(html.contains(r#"<main id="main">"#));
    assert!(html.contains(r##"href="#main""##));
    assert!(html.contains("Rust &lt;HTML&gt; &amp; a real stylesheet."));
    assert!(html.contains(&stylesheet_url));

    let response = app.oneshot(request(&stylesheet_url)).await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/css; charset=utf-8");
    assert!(
        response.headers()[CACHE_CONTROL]
            .to_str()?
            .contains("immutable")
    );
    let css = to_bytes(response.into_body(), 1024 * 1024).await?;
    assert_eq!(css.as_ref(), stylesheet.bytes.as_ref());
    println!("Verified HTML and {stylesheet_url} through in-memory requests.");
    Ok(())
}
```

```sh
cargo run --locked --manifest-path site/downloads/stucco/axum-page/Cargo.toml
```

A successful run prints the checked stylesheet URL. It sends requests through Tower in memory and does not start a network listener.

Release evidence: [documentation, lines 58–82](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/docs/guide/getting-started.md#L58-L82) · [api, lines 110–111](https://docs.rs/crate/stucco/0.2.1/source/src/lib.rs) · [example, lines 1–25](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/examples/orders/tests/http.rs#L1-L25) · [api, lines 121–124](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/crates/stucco-core/src/bundle.rs#L121-L124) · [api, lines 163–168](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/crates/stucco-core/src/bundle.rs#L163-L168) · [manifest, lines 20–28](https://github.com/mcaveniathor/stucco/blob/303ee2433ce3c05d0af9c4b79a4e09958d374f92/Cargo.toml#L20-L28).

## What was validated

All eight projects compiled against their exact primary crate releases with recorded dependency lockfiles. The clap recipes also passed six parser tests; the three axum executables passed their response assertions. The two Stucco recipes passed assertions for rendered HTML, escaping, main landmarks, asset output, and in-memory HTML/CSS responses. These checks establish the behavior shown here, not comprehensive library coverage.

The collector verified all three published crate archives and read repository snapshots pinned by the published VCS metadata. The clap and axum pilots each retained 48 evidence sources; Stucco retained 27 before the supplemental API and license files described above were added. All bundles record coverage limits. Source citations refer to those frozen inputs; inspect bundle warnings when creating your own guidance.

After installing a generated plugin, use `cargo agents sync` in your consumer workspace. Skills and MCP registration use exact release predicates. The SessionStart hook warns when the consumer's lockfile no longer matches. See the [getting started guide](getting-started.html) for installation and [command reference](reference.html) for refresh and retrieval.
