# Examples you can inspect and run

These twelve recipes were checked with Rust 1.99.1. Each project pins its primary crate and includes the lockfile used for validation. The versions below are **validated snapshots**, not a promise that they are still the latest release.

| Crate | Version | Recipes | Additional behavior checks |
| --- | --- | --- | --- |
| clap | 4.6.7 | Derive arguments, subcommands, fallible validation | Six parser tests passed |
| axum | 0.8.9 | Typed routes, shared state, JSON bodies | Three in-memory requests passed |
| Stucco | 0.3.0 | Static pages, fragments, tables, forms, themes, and status displays | Six executables with rendering, asset, validation, and HTTP assertions passed |

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


## Stucco 0.3.0

This documentation site is built with Stucco 0.3.0. The six recipes below cover static asset export, Axum fragments, derived tables, validation feedback, server-selected themes, and native status/detail components.

### Install the Stucco plugin

With Symposiast and Symposium configured, run from this checkout:

```sh
symposiast install plugins/stucco/bundle
cd /path/to/consumer
cargo agents sync
```

Guidance and MCP retrieval match Stucco **0.3.0** exactly. The version hook warns about other releases. Restart the consuming MCP session after installation and inspect the [plugin guide](https://github.com/mcaveniathor/symposiast/blob/main/plugins/stucco/README.md).

The bundle contains 33 frozen sources, including both upstream license texts, at release commit `07335e73dfb22aba2ce89487bcee634a24c2080e`. Its 57 inventoried documentation examples have 14 adapted, 13 supporting, and 30 skipped decisions with reasons. The generated plugin exposes that complete coverage report. This is coverage of the collected evidence, not the entire crate.

Every recipe has a recorded compilation check and a pinned consumer lockfile. Local executable assertions are recorded separately in the [validation notes](https://github.com/mcaveniathor/symposiast/blob/main/docs/validation.md). They do not establish browser behavior or application storage integration.

### Export a static page with its assets

Write a complete page and its hashed theme assets to an optional output directory (default stucco-example). Assertions check escaped text, the main landmark, the skip link, and exact exported bytes. Serve the output directory at the URL root.

[Cargo.toml](examples/stucco/static-page/Cargo.toml) · [Cargo.lock](examples/stucco/static-page/Cargo.lock) · [src/main.rs](examples/stucco/static-page/src/main.rs)

```toml
[dependencies.stucco]
default-features = false
features = [
    "layout",
    "typography",
]
version = "=0.3.0"

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("stucco-example"));
    let bundle = Bundle::new(Preset::Slate);
    let html = Page::new(&bundle, "Hello, Stucco")
        .main(
            Container::new().child(
                Stack::new()
                    .child(Heading::new(1, "Hello, Stucco"))
                    .child(Text::new("A <careful> welcome & a fresh start.")),
            ),
        )
        .render();
    assert!(html.contains(r#"<main id="main">"#));
    assert!(html.contains(r##"href="#main""##));
    assert!(html.contains("A &lt;careful&gt; welcome &amp; a fresh start."));
    assert!(html.contains(bundle.stylesheet_url()));
    for path in bundle.paths() {
        let relative = Path::new(path.trim_start_matches('/'));
        assert!(
            relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        );
        let destination = output.join(relative);
        fs::create_dir_all(destination.parent().expect("asset has a parent"))?;
        let asset = bundle.get(&path).expect("listed asset exists");
        fs::write(&destination, &*asset.bytes)?;
        assert_eq!(fs::read(&destination)?, asset.bytes.as_ref());
    }
    fs::write(output.join("index.html"), &html)?;
    assert_eq!(fs::read_to_string(output.join("index.html"))?, html);
    println!(
        "Wrote {} with matching theme assets; serve this directory at /.",
        output.display()
    );
    Ok(())
}
```

```sh
cargo run --locked --manifest-path site/downloads/stucco/static-page/Cargo.toml -- /tmp/stucco-preview
```

Release evidence: [readme, lines 8–18](https://docs.rs/crate/stucco/0.3.0/source/README.md) · [documentation, lines 179–195](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L179-L195) · [manifest, lines 87–94](https://docs.rs/crate/stucco/0.3.0/source/Cargo.toml) · [readme, lines 7–19](https://docs.rs/crate/stucco/0.3.0/source/README.md) · [readme, lines 38–53](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/README.md#L38-L53) · [api, lines 65–78](https://docs.rs/crate/stucco/0.3.0/source/src/lib.rs) · [documentation, lines 38–52](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/getting-started.md#L38-L52) · [documentation, lines 183–195](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L183-L195).

### Serve full pages and a named fragment

Exercise full-page, named-fragment, unknown-target fallback, escaping, and stylesheet responses with in-memory requests. Pass --serve to listen on 127.0.0.1:3000 after the checks.

[Cargo.toml](examples/stucco/axum-fragments/Cargo.toml) · [Cargo.lock](examples/stucco/axum-fragments/Cargo.lock) · [src/main.rs](examples/stucco/axum-fragments/src/main.rs)

```toml
[dependencies.axum]
default-features = true
features = []
version = "0.8"

[dependencies.stucco]
default-features = true
features = ["axum"]
version = "=0.3.0"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1"

[dependencies.tower]
default-features = true
features = ["util"]
version = "0.5"

[package]
edition = "2024"
name = "recipe-axum-fragments"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Query,
    http::{
        Request, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE, VARY},
    },
    response::Response,
    routing::get,
};
use std::collections::HashMap;
use stucco::prelude::*;
use tower::ServiceExt;

fn greeting(name: &str) -> impl Render {
    el::div()
        .id("greeting")
        .child(Text::new(format!("Hello, {name}!")))
}

async fn index(page: PageCx, Query(params): Query<HashMap<String, String>>) -> Response {
    let name = params.get("q").map(String::as_str).unwrap_or("visitor");
    page.respond()
        .fragment("greeting", || greeting(name))
        .page(|| {
            page.title("Greeting").main(Stack::of((
                Heading::new(1, "Greeting"),
                Form::get("/").child(Stack::of((
                    Field::new("Name", Input::text("q").value(name)),
                    Button::new("Greet").submit(),
                ))),
                greeting(name),
            )))
        })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = Bundle::new(Preset::Slate);
    let stylesheet_url = bundle.stylesheet_url().to_owned();
    let stylesheet = bundle
        .get(&stylesheet_url)
        .expect("theme stylesheet exists");
    // Call stucco last to attach assets, middleware, and PageCx to all routes.
    let app = Router::new().route("/", get(index)).stucco(bundle);
    for target in [None, Some("greeting"), Some("unknown")] {
        let mut request = Request::builder().uri("/?q=%3CAda%3E");
        if let Some(target) = target {
            request = request
                .header("Stucco-Request", "fragment")
                .header("Stucco-Target", target);
        }
        let response = app.clone().oneshot(request.body(Body::empty())?).await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[CONTENT_TYPE], "text/html; charset=utf-8");
        assert_eq!(response.headers()[VARY], "Stucco-Request, Stucco-Target");
        if target == Some("greeting") {
            assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        }
        let html = String::from_utf8(to_bytes(response.into_body(), 1024 * 1024).await?.to_vec())?;
        assert!(html.contains("Hello, &lt;Ada&gt;!"));
        assert_eq!(html.contains("<html"), target != Some("greeting"));
        assert_eq!(html.contains("<form"), target != Some("greeting"));
    }
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&stylesheet_url)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/css; charset=utf-8");
    assert!(
        response.headers()[CACHE_CONTROL]
            .to_str()?
            .contains("immutable")
    );
    assert_eq!(
        to_bytes(response.into_body(), 1024 * 1024).await?.as_ref(),
        stylesheet.bytes.as_ref()
    );
    println!("Verified full pages, fragments, fallback, escaping, and stylesheet bytes.");
    if std::env::args().any(|argument| argument == "--serve") {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
        axum::serve(listener, app).await?;
    }
    Ok(())
}
```

```sh
cargo run --locked --manifest-path site/downloads/stucco/axum-fragments/Cargo.toml
```

The default run completes after in-memory HTTP checks. Append `-- --serve` to start the demo server at `http://127.0.0.1:3000`.

Release evidence: [readme, lines 65–66](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/README.md#L65-L66) · [example, lines 6–63](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/examples/hello/src/main.rs#L6-L63) · [documentation, lines 9–26](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/enhancement.md#L9-L26) · [readme, lines 72–86](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/README.md#L72-L86) · [documentation, lines 13–22](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/enhancement.md#L13-L22) · [documentation, lines 62–76](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/getting-started.md#L62-L76) · [documentation, lines 147–160](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L147-L160).

### Derive columns and format stored cents

Render table markup from derived columns with separate numeric and display values. This has no backend capabilities or query execution; embed the table in a Page for themed document output.

[Cargo.toml](examples/stucco/derived-table/Cargo.toml) · [Cargo.lock](examples/stucco/derived-table/Cargo.lock) · [src/main.rs](examples/stucco/derived-table/src/main.rs)

```toml
[dependencies.stucco]
default-features = false
features = ["derive"]
version = "=0.3.0"

[package]
edition = "2024"
name = "recipe-derived-table"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use stucco::prelude::*;
use stucco::{CollectionPage, to_html};

#[derive(Columns)]
struct Order {
    #[col(label = "ID", sortable)]
    id: u64,
    #[col(sortable, searchable)]
    customer: String,
    #[col(key = "total", label = "Total", value = dollars, display = money)]
    total_cents: u64,
}

fn dollars(order: &Order) -> f64 {
    order.total_cents as f64 / 100.0
}

fn money(order: &Order) -> String {
    format!(
        "${}.{:02}",
        order.total_cents / 100,
        order.total_cents % 100
    )
}

fn main() {
    let page = CollectionPage {
        rows: vec![Order {
            id: 7,
            customer: "<Ada>".into(),
            total_cents: 4800,
        }],
        next: None,
        prev: None,
        total: None,
    };
    let table = DataTable::from_page(&page, "Orders").columns(Order::columns());
    let html = to_html(&table);
    assert!(html.contains("&lt;Ada&gt;"));
    assert!(html.contains("$48.00"));
    println!("{html}");
}
```

```sh
cargo run --locked --manifest-path site/downloads/stucco/derived-table/Cargo.toml
```

Release evidence: [manifest, lines 77–80](https://docs.rs/crate/stucco/0.3.0/source/Cargo.toml) · [test, lines 4–34](https://docs.rs/crate/stucco/0.3.0/source/tests/derive.rs) · [test, lines 50–74](https://docs.rs/crate/stucco/0.3.0/source/tests/derive.rs) · [readme, lines 160–173](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/README.md#L160-L173) · [documentation, lines 15–35](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/collections.md#L15-L35) · [documentation, lines 45–67](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/collections.md#L45-L67) · [documentation, lines 110–139](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L110-L139).

### Render validation errors from returned form state

Demonstrate validation and bound error markup without an HTTP server. A deployed POST flow must supply its handler, CSRF setup and issued token, and a 422 response on validation failure.

[Cargo.toml](examples/stucco/bound-validation/Cargo.toml) · [Cargo.lock](examples/stucco/bound-validation/Cargo.lock) · [src/main.rs](examples/stucco/bound-validation/src/main.rs)

```toml
[dependencies.stucco]
default-features = false
features = ["forms"]
version = "=0.3.0"

[package]
edition = "2024"
name = "recipe-bound-validation"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use stucco::prelude::*;
use stucco::{FormState, Validator, to_html};

fn validate(state: FormState) -> Result<String, FormState> {
    let mut validator = Validator::new(state);
    let customer = validator
        .text("customer")
        .required("Enter a customer name")
        .max_chars(80, "Use 80 characters or fewer")
        .get();
    validator.finish(|| customer)
}

fn main() {
    let submitted = FormState::new().with_value("customer", "");
    match validate(submitted) {
        Ok(_) => panic!("an empty customer name must fail validation"),
        Err(state) => {
            let form = Form::post("/customers")
                .child(ErrorSummary::new(&state).field("customer", "customer-name"))
                .child(
                    Field::new("Customer", Input::text("customer").id("customer-name"))
                        .required()
                        .bind(&state),
                )
                .child(Button::new("Save").submit());
            let html = to_html(&form);
            assert!(html.contains("Enter a customer name"));
            assert!(html.contains("aria-invalid=\"true\""));
            assert!(html.contains("customer-name"));
            println!("{html}");
        }
    }
}
```

```sh
cargo run --locked --manifest-path site/downloads/stucco/bound-validation/Cargo.toml
```

Release evidence: [documentation, lines 44–99](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/forms.md#L44-L99) · [example, lines 97–107](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/examples/orders/src/form.rs#L97-L107) · [documentation, lines 83–101](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L83-L101) · [manifest, lines 82–85](https://docs.rs/crate/stucco/0.3.0/source/Cargo.toml) · [documentation, lines 9–26](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/forms.md#L9-L26) · [documentation, lines 48–64](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/forms.md#L48-L64) · [documentation, lines 74–95](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/forms.md#L74-L95) · [documentation, lines 75–102](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L75-L102).

### Render an inline document with a server-selected scheme

Render a standalone document with an explicit dark scheme and inline assets. This example uses basic elements and no component behaviors; inline output cannot receive fragments.

[Cargo.toml](examples/stucco/inline-server-theme/Cargo.toml) · [Cargo.lock](examples/stucco/inline-server-theme/Cargo.lock) · [src/main.rs](examples/stucco/inline-server-theme/src/main.rs)

```toml
[dependencies.stucco]
default-features = false
features = []
version = "=0.3.0"

[package]
edition = "2024"
name = "recipe-inline-server-theme"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use stucco::theme::{Preset, Scheme};
use stucco::{Bundle, Delivery, Page, el};

fn main() {
    let bundle = Bundle::new(Preset::Slate);
    let html = Page::new(&bundle, "Account")
        .server_theme(Some(Scheme::Dark))
        .delivery(Delivery::Inline)
        .body(el::main().child(el::h1().text("Account")))
        .render();
    assert!(html.contains("data-theme=\"dark\""));
    assert!(html.contains("<style"));
    assert!(!html.contains("rel=\"stylesheet\""));
    assert!(html.contains("<h1>Account</h1>"));
    println!("{html}");
}
```

```sh
cargo run --locked --manifest-path site/downloads/stucco/inline-server-theme/Cargo.toml
```

Release evidence: [api, lines 19–24](https://docs.rs/crate/stucco/0.3.0/source/src/lib.rs) · [documentation, lines 48–50](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/enhancement.md#L48-L50) · [documentation, lines 333–350](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L333-L350) · [api, lines 5–13](https://docs.rs/crate/stucco/0.3.0/source/src/lib.rs) · [documentation, lines 339–348](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L339-L348) · [documentation, lines 9–13](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/theming.md#L9-L13).

### Render status and details with native components

Render a standalone order page demonstrating all eleven new display, navigation, progress and disclosure components. Replace sample values and navigation URLs with application data and routes.

[Cargo.toml](examples/stucco/status-details/Cargo.toml) · [Cargo.lock](examples/stucco/status-details/Cargo.lock) · [src/main.rs](examples/stucco/status-details/src/main.rs)

```toml
[dependencies.stucco]
default-features = true
features = []
version = "=0.3.0"

[package]
edition = "2024"
name = "recipe-status-details"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use stucco::Delivery;
use stucco::prelude::*;

fn main() {
    let bundle = Bundle::new(Preset::Slate);
    let content = Stack::new()
        .child(
            PageHeader::new("Order 1042").breadcrumbs(
                Breadcrumbs::new()
                    .link("Orders", "/orders")
                    .current("Order 1042"),
            ),
        )
        .child(Cluster::of((
            Avatar::new("Ada Lovelace").decorative(),
            Text::new("Ada Lovelace"),
        )))
        .child(
            DescriptionList::new()
                .item("Status", Badge::new("Paid").tone(Tone::Success))
                .item("Total", "$128.00"),
        )
        .child(
            Stat::new("Orders", "128")
                .change("12%", Trend::Up)
                .change_tone(Tone::Success)
                .context("vs last week"),
        )
        .child(
            Steps::new("Checkout")
                .step("Cart", "/checkout/cart")
                .step("Delivery", "/checkout/delivery")
                .step_disabled("Payment")
                .current(1),
        )
        .child(Progress::new("Import", 42.0, 100.0).show_value())
        .child(
            Meter::new("Storage", 7.2, 0.0, 10.0)
                .low(3.0)
                .high(8.0)
                .optimum(1.0)
                .value_text("7.2 of 10 GB"),
        )
        .child(
            Callout::new(Tone::Info)
                .title("Preparing for dispatch")
                .child(Text::new("Your order will ship within two working days.")),
        )
        .child(
            Disclosure::new("Shipping details")
                .heading(2)
                .child(Text::new("Tracked delivery is included.")),
        )
        .child(
            Accordion::new()
                .heading(2)
                .item_open("Shipping", Text::new("Arrives by Friday."))
                .item("Returns", Text::new("Return within 30 days."))
                .exclusive(),
        );
    let html = Page::new(&bundle, "Order 1042")
        .delivery(Delivery::Inline)
        .main(content)
        .render();
    for marker in [
        "<progress",
        "<meter",
        "<details",
        "<summary",
        "Paid",
        "Order 1042",
        "$128.00",
    ] {
        assert!(html.contains(marker), "missing {marker}");
    }
    println!("{html}");
}
```

```sh
cargo run --locked --manifest-path site/downloads/stucco/status-details/Cargo.toml
```

Release evidence: [documentation, lines 106–202](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/components.md#L106-L202) · [documentation, lines 36–69](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L36-L69) · [documentation, lines 48–50](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/enhancement.md#L48-L50) · [api, lines 82–98](https://docs.rs/crate/stucco/0.3.0/source/src/lib.rs) · [documentation, lines 29–45](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/components.md#L29-L45) · [documentation, lines 110–126](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/components.md#L110-L126) · [documentation, lines 139–152](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/components.md#L139-L152) · [documentation, lines 162–172](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/components.md#L162-L172) · [documentation, lines 182–198](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/components.md#L182-L198) · [documentation, lines 40–67](https://github.com/mcaveniathor/stucco/blob/07335e73dfb22aba2ce89487bcee634a24c2080e/docs/guide/recipes.md#L40-L67).
