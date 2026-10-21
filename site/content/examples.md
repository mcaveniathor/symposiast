# Examples you can inspect and run

These 32 recipes compile against exact published crate versions with recorded consumer lockfiles. The versions are validated snapshots. Compilation means `cargo check --locked --all-targets` passed; runtime checks are recorded separately in the [validation notes](https://github.com/mcaveniathor/symposiast/blob/main/docs/validation.md).

| Crate | Version | Recipes | Analysis source |
| --- | --- | --- | --- |
| clap | 4.6.7 | 8 | Codex adapter |
| axum | 0.8.9 | 13 | Codex adapter |
| Bevy | 0.20.0 | 5 | Codex adapter |
| Stucco | 0.3.0 | 6 | Imported analysis |

## Generate recipes from release findings

Symposiast asks its analysis agent to produce findings and complete consumer recipes from the collected evidence. It inventories documentation code blocks, requires an adapted/supporting/skipped decision for each one, validates citations and feature flags, then compiles the generated recipes and renders the plugin. No hand-written recipe input is required.

```sh
symposiast build clap --version 4.6.7 --agent codex \
  --output .symposiast/clap-generated
symposiast build axum --version 0.8.9 --agent codex \
  --output .symposiast/axum-generated
symposiast build bevy --agent codex \
  --output .symposiast/bevy-generated
```

Omit `--version` to select the latest stable, non-yanked release. Generation requires an authenticated agent CLI; a new run may produce different recipes. The checked-in bundles preserve the actual findings, recipes, coverage decisions, and compilation records used by this page. See [agent adapters](adapters.html) for setup and the structured import alternative.

## Use the downloadable projects

Clone the [repository](https://github.com/mcaveniathor/symposiast), or save a recipe's linked `Cargo.toml`, `Cargo.lock`, and `src/main.rs` with the same directory structure. Each recipe includes its verification command. No analysis agent or Symposium installation is needed to compile a saved project. Read its main function before running: a server may listen until stopped, and a graphical Bevy application needs the corresponding display, driver, and platform libraries.

## clap 4.6.7

Symposiast generated this guidance through its Codex adapter from frozen release evidence. The bundle contains 50 sources and 8 recipes. Its 122 inventoried documentation examples have 14 adapted, 6 supporting, and 102 skipped decisions. Inspect the [complete inventory and coverage reasons](examples/clap/coverage.json). Standalone upstream example programs are also evidence, but are not counted as documentation code blocks. All recipes passed compilation after analysis; the agent's original pre-verification summary is preserved in the bundle.

Clap uses external includes in its rustdoc. The inventory preserves literal fragments around these boundaries; skipped fragments do not establish coverage of the included examples.

Install the [checked-in bundle](https://github.com/mcaveniathor/symposiast/tree/main/plugins/clap) with `symposiast install plugins/clap/bundle`, then run `cargo agents sync` in the consumer workspace. Skills and MCP retrieval target 4.6.7 exactly. Restart the consuming MCP session after installation.

### Findings used by the generated recipes

- **Follow the release manifest and enforced std requirement** (documented): Use Rust 1.85 or newer for clap 4.6.7. Keep std enabled, including when disabling default features: the implementation rejects builds without it. The overview's MSRV 1.74 and feature reference's description of std as unused are stale relative to the manifest and compile-time guard. [manifest, lines 12–16](https://docs.rs/crate/clap/4.6.7/source/Cargo.toml) · [changelog, lines 69–73](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L69-L73) · [api, lines 29–30](https://docs.rs/crate/clap/4.6.7/source/src/lib.rs) · [api, lines 14–15](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/lib.rs#L14-L15) · [api, lines 5–7](https://docs.rs/crate/clap/4.6.7/source/src/_features.rs).
- **Choose the API and enable only the needed optional features** (documented): Prefer derive for a fixed CLI whose values should become Rust fields. Choose builder when inspecting argument order or value sources, or when constructing the interface at runtime. Enable derive explicitly; cargo, env, string, and wrap\_help are separate opt-ins for their respective capabilities. [api, lines 74–88](https://docs.rs/crate/clap/4.6.7/source/src/_faq.rs) · [manifest, lines 115–127](https://docs.rs/crate/clap/4.6.7/source/Cargo.toml) · [api, lines 15–23](https://docs.rs/crate/clap/4.6.7/source/src/_features.rs).
- **Match argument actions, field types, and getters** (documented): In builder code, use SetTrue for switches, Count for repeated flags, Set for a single value, and Append for repeated values. Read these with get\_flag, get\_count, get\_one, and get\_many respectively. Derive infers bool as a switch, Option&lt;T&gt; as optional, T as required unless defaulted, and Vec&lt;T&gt; as repeated. [changelog, lines 1280–1294](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1280-L1294) · [changelog, lines 1300–1308](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1300-L1308) · [documentation, lines 320–335](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs).
- **Separate effective values from explicit occurrences** (inferred): Do not use contains\_id to decide whether a boolean switch was supplied: the supplied tests show that its default false value also creates an entry. Use get\_flag for the effective boolean and value\_source when provenance matters. Repeated Set or SetTrue arguments conflict by default; choose Append, Count, or args\_override\_self(true) deliberately. [test, lines 26–39](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/tests/builder/action.rs#L26-L39) · [test, lines 81–103](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/tests/builder/action.rs#L81-L103) · [example, lines 68–74](https://docs.rs/crate/clap/4.6.7/source/examples/find.rs).
- **Set named-option requiredness explicitly** (documented): arg!(--config &lt;FILE&gt;) requires a value when the option appears, but does not require the option itself. Add .required(true) when config must be supplied. The macro example's help text calls the file required without setting this requirement; do not copy that implication. [changelog, lines 1311–1311](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1311-L1311) · [test, lines 11–18](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/tests/macros.rs#L11-L18) · [api, lines 516–526](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L516-L526).
- **Check both configuration and parsing behavior** (documented): Add a test calling Command::debug\_assert, using CommandFactory::command for a derived parser. Use explicit argument iterators and fallible parsing to check valid inputs and rejected inputs without exiting the test process. Configuration checks complement behavioral assertions. [documentation, lines 230–234](https://docs.rs/crate/clap/4.6.7/source/src/_tutorial.rs) · [changelog, lines 1254–1266](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1254-L1266) · [changelog, lines 1268–1278](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1268-L1278) · [api, lines 70–78](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L70-L78).
- **Control short and long help deliberately** (documented): Use doc comments for help summaries and a blank line followed by details for long help. -h selects short text and --help selects long text. Explicit attributes override inferred documentation; clear an unwanted long description with #\[command(long\_about = None)\]. [documentation, lines 348–353](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 382–388](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 413–438](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs).
- **Do not infer flags from help comments** (inferred): The two Foo help examples are not equivalent CLI declarations: the first has #\[arg(short)\] and the second does not. Doc comments supply help text, not flag spelling. Add short or long explicitly when adapting a documented field into an option. [documentation, lines 355–378](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 256–261](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs).
- **Opt into deferred subcommand construction** (documented): In 4.6.7, opt into lazy subcommand argument construction with #\[command(defer = true)\] on a Parser or Subcommand enum. It defaults to false without unstable-v5. Put parent-help metadata on enum variants, configure nested enums separately, and call Command::build before inspecting deferred arguments. Parsing builds selected commands automatically. [changelog, lines 27–31](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L27-L31) · [documentation, lines 164–177](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs).
- **Reuse derived types inside builder applications** (documented): Combine builder and derive with Args::augment\_args or Subcommand::augment\_subcommands, then convert the resulting matches through FromArgMatches. This lets reusable derived argument sets participate in a builder-owned interface. [documentation, lines 451–464](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 475–482](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [api, lines 127–165](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L127-L165) · [api, lines 262–269](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L262-L269).
- **Read application package metadata with the cargo feature** (documented): Enable cargo for command! and crate\_\* metadata macros. They read the consuming package's Cargo metadata at compile time. crate\_name! uses CARGO\_PKG\_NAME; use the appropriate Cargo environment variable directly if the binary or crate name differs. [documentation, lines 44–45](https://docs.rs/crate/clap/4.6.7/source/src/_tutorial.rs) · [api, lines 18–19](https://docs.rs/crate/clap/4.6.7/source/src/_features.rs) · [api, lines 80–105](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L80-L105) · [api, lines 129–147](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L129-L147).
- **Preserve argument boundaries and explicit escapes** (documented): Pass already separated arguments to clap; parsing a shell command string requires a separate tokenizer. Use last(true) for a positional accessible only after --. Use allow\_hyphen\_values or allow\_negative\_numbers only when the argument's grammar calls for them. [api, lines 15–34](https://docs.rs/crate/clap/4.6.7/source/src/_concepts.rs) · [api, lines 59–73](https://docs.rs/crate/clap/4.6.7/source/src/_concepts.rs).
- **Model absent, bare, and explicit option values** (inferred): For a color-style option, combine num\_args(0..=1), require\_equals, a default value, and default\_missing\_value to distinguish omission, a bare --color, and --color=never. ValueEnum constrains the accepted values and permits default\_value\_t without a Display implementation. [example, lines 33–42](https://docs.rs/crate/clap/4.6.7/source/examples/git-derive.rs) · [example, lines 25–31](https://docs.rs/crate/clap/4.6.7/source/examples/git.rs) · [api, lines 281–286](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L281-L286).

### Build a typed CLI with defaults and configuration checks

Parse a required name and a typed default count from explicit inputs, with a configuration test. In an application entry point, Cli::parse() reads process arguments and handles CLI exits.

[Cargo.toml](examples/clap/derive_basics/Cargo.toml) · [Cargo.lock](examples/clap/derive_basics/Cargo.lock) · [src/main.rs](examples/clap/derive_basics/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-derive_basics"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{CommandFactory, Parser};

/// Greet a person.
#[derive(Debug, Parser)]
#[command(name = "greet", version, long_about = None)]
struct Cli {
    /// Name of the person to greet.
    #[arg(short, long)]
    name: String,

    /// Number of greetings.
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() -> Result<(), clap::Error> {
    Cli::command().debug_assert();
    let cli = Cli::try_parse_from(["greet", "--name", "Ada"])?;
    assert_eq!(cli.count, 1);
    assert_eq!(cli.name, "Ada");
    assert!(Cli::try_parse_from(["greet"]).is_err());
    assert!(Cli::try_parse_from(["greet", "--name", "Ada", "--count", "oops"]).is_err());
    for _ in 0..cli.count {
        println!("Hello {}!", cli.name);
    }
    Ok(())
}

#[test]
fn verify_cli() {
    Cli::command().debug_assert();
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/clap/derive_basics/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/clap/derive_basics/Cargo.toml`. This is a finite executable.

Release evidence: [example, lines 1–22](https://docs.rs/crate/clap/4.6.7/source/examples/demo.rs) · [changelog, lines 1255–1266](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1255-L1266) · [documentation, lines 526–537](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [api, lines 70–78](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L70-L78) · [manifest, lines 115–127](https://docs.rs/crate/clap/4.6.7/source/Cargo.toml).

### Convert builder matches into application data

Declare matching builder arguments before converting ArgMatches into an application struct. The documentation's derive cfg guards are unnecessary here because no clap derive macro is used.

[Cargo.toml](examples/clap/builder_context/Cargo.toml) · [Cargo.lock](examples/clap/builder_context/Cargo.lock) · [src/main.rs](examples/clap/builder_context/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = []
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-builder_context"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{Arg, ArgAction, ArgMatches, Command};

#[derive(Debug)]
struct Context {
    name: String,
    debug: bool,
}

impl From<ArgMatches> for Context {
    fn from(matches: ArgMatches) -> Self {
        Self {
            name: matches.get_one::<String>("name").expect("required name").clone(),
            debug: matches.get_flag("debug"),
        }
    }
}

fn cli() -> Command {
    Command::new("app")
        .arg(Arg::new("name").long("name").required(true))
        .arg(Arg::new("debug").long("debug").action(ArgAction::SetTrue))
        .arg(Arg::new("option").long("option"))
}

fn main() -> Result<(), clap::Error> {
    cli().debug_assert();
    let matches = cli().try_get_matches_from([
        "app", "--name", "Ada", "--debug", "--option", "value",
    ])?;
    assert_eq!(matches.get_one::<String>("option").map(String::as_str), Some("value"));
    let context = Context::from(matches);
    assert_eq!(context.name, "Ada");
    assert!(context.debug);

    let defaults = cli().try_get_matches_from(["app", "--name", "Ada"])?;
    assert!(!defaults.get_flag("debug"));
    assert_eq!(defaults.get_one::<String>("option"), None);
    assert!(cli().try_get_matches_from(["app"]).is_err());
    println!("{context:?}");
    Ok(())
}

#[test]
fn verify_cli() {
    cli().debug_assert();
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/clap/builder_context/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/clap/builder_context/Cargo.toml`. This is a finite executable.

Release evidence: [changelog, lines 1269–1278](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1269-L1278) · [changelog, lines 1288–1294](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1288-L1294) · [api, lines 136–143](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L136-L143) · [api, lines 148–164](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L148-L164) · [api, lines 172–179](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L172-L179) · [api, lines 184–200](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L184-L200) · [documentation, lines 120–123](https://docs.rs/crate/clap/4.6.7/source/src/_tutorial.rs) · [test, lines 81–103](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/tests/builder/action.rs#L81-L103).

### Generate help from comments and explicit overrides

Render short and long help, override inferred text with attributes, and explicitly retain option spelling when using doc comments.

[Cargo.toml](examples/clap/help_text/Cargo.toml) · [Cargo.lock](examples/clap/help_text/Cargo.lock) · [src/main.rs](examples/clap/help_text/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-help_text"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{CommandFactory, Parser};

/// Inspect a report.
///
/// Detailed command documentation that is replaced by explicit metadata.
#[derive(Parser)]
#[command(name = "report", about = "Inspect a report", long_about = None)]
struct Cli {
    /// Report name.
    #[arg(short, long, help = "Select a report")]
    name: String,

    /// Show details.
    ///
    /// Include individual records in the report output.
    #[arg(long)]
    verbose: bool,
}

fn main() -> Result<(), clap::Error> {
    Cli::command().debug_assert();
    let cli = Cli::try_parse_from(["report", "-n", "daily", "--verbose"])?;
    assert_eq!(cli.name, "daily");
    assert!(cli.verbose);

    let mut command = Cli::command();
    let short = command.render_help().to_string();
    let long = command.render_long_help().to_string();
    assert!(short.contains("Select a report"));
    assert!(!short.contains("Include individual records"));
    assert!(long.contains("Include individual records"));
    assert!(!long.contains("Detailed command documentation"));
    println!("{long}");
    Ok(())
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/clap/help_text/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/clap/help_text/Cargo.toml`. This is a finite executable.

Release evidence: [documentation, lines 355–364](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 369–378](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 394–411](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 413–422](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 382–388](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [changelog, lines 1411–1411](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1411-L1411).

### Compose typed subcommands with deferred construction

Compose reusable Args, a required subcommand, and an enum-valued option. Opt into the new 4.6.7 defer attribute and build before inspecting deferred arguments.

[Cargo.toml](examples/clap/deferred_subcommands/Cargo.toml) · [Cargo.lock](examples/clap/deferred_subcommands/Cargo.lock) · [src/main.rs](examples/clap/deferred_subcommands/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-deferred_subcommands"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "jobs")]
struct Cli {
    #[command(flatten)]
    logging: Logging,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Args)]
struct Logging {
    /// Increase verbosity.
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Debug, Subcommand)]
#[command(defer = true)]
enum Commands {
    /// Run a named job.
    Run(RunArgs),
}

#[derive(Debug, Args)]
struct RunArgs {
    name: String,
    #[arg(long, value_enum, default_value_t = Mode::Fast)]
    mode: Mode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Mode {
    Fast,
    Careful,
}

fn main() -> Result<(), clap::Error> {
    let mut command = Cli::command();
    command.build();
    let run = command.find_subcommand("run").expect("registered subcommand");
    assert!(run.get_arguments().any(|arg| arg.get_id() == "mode"));
    command.debug_assert();

    let cli = Cli::try_parse_from(["jobs", "-vv", "run", "daily", "--mode", "careful"])?;
    assert_eq!(cli.logging.verbose, 2);
    let Commands::Run(run) = cli.command;
    assert_eq!(run.name, "daily");
    assert_eq!(run.mode, Mode::Careful);
    assert!(Cli::try_parse_from(["jobs", "run", "daily", "--mode", "unknown"]).is_err());
    println!("{run:?}");
    Ok(())
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/clap/deferred_subcommands/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/clap/deferred_subcommands/Cargo.toml`. This is a finite executable.

Release evidence: [documentation, lines 23–85](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 157–160](https://docs.rs/crate/clap/4.6.7/source/src/_derive/_tutorial.rs) · [documentation, lines 164–177](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 197–204](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 266–272](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 320–335](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 117–119](https://docs.rs/crate/clap/4.6.7/source/src/_derive/_tutorial.rs) · [api, lines 70–78](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L70-L78).

### Use Cargo metadata and concise argument macros

Combine Cargo metadata macros with arg! declarations. The no\_run metadata snippets are adapted to explicit inputs, and config requiredness is set explicitly.

[Cargo.toml](examples/clap/cargo_metadata_and_args/Cargo.toml) · [Cargo.lock](examples/clap/cargo_metadata_and_args/Cargo.lock) · [src/main.rs](examples/clap/cargo_metadata_and_args/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["cargo"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-cargo_metadata_and_args"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{arg, command, crate_authors, crate_description, crate_name, crate_version, Command};

fn cli() -> Command {
    command!().args([
        arg!(--config <FILE> "Configuration file").required(true),
        arg!(-d --debug ... "Increase debug verbosity"),
        arg!([input] "Optional input file"),
    ])
}

fn main() -> Result<(), clap::Error> {
    // The explicit form shows the metadata macros used by command!.
    Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .try_get_matches_from(["app"])?;

    cli().debug_assert();
    let matches = cli().try_get_matches_from(["app", "--config", "file.toml"])?;
    assert_eq!(matches.get_one::<String>("config").map(String::as_str), Some("file.toml"));
    assert_eq!(matches.get_count("debug"), 0);
    assert_eq!(matches.get_one::<String>("input"), None);

    let repeated = cli().try_get_matches_from([
        "app", "--config", "file.toml", "-dd", "input.txt",
    ])?;
    assert_eq!(repeated.get_count("debug"), 2);
    assert_eq!(repeated.get_one::<String>("input").map(String::as_str), Some("input.txt"));
    assert!(cli().try_get_matches_from(["app"]).is_err());
    Ok(())
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/clap/cargo_metadata_and_args/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/clap/cargo_metadata_and_args/Cargo.toml`. This is a finite executable.

Release evidence: [api, lines 6–13](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L6-L13) · [api, lines 33–40](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L33-L40) · [api, lines 64–71](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L64-L71) · [api, lines 94–100](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L94-L100) · [api, lines 124–128](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L124-L128) · [api, lines 513–527](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L513-L527) · [api, lines 129–147](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/macros.rs#L129-L147) · [changelog, lines 1311–1311](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1311-L1311) · [test, lines 11–18](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/tests/macros.rs#L11-L18) · [changelog, lines 1300–1304](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/CHANGELOG.md#L1300-L1304) · [documentation, lines 44–45](https://docs.rs/crate/clap/4.6.7/source/src/_tutorial.rs).

### Mix builder configuration with derived argument types

Attach derived arguments and subcommands to a builder command and recover their typed values through FromArgMatches.

[Cargo.toml](examples/clap/builder_derive_interop/Cargo.toml) · [Cargo.lock](examples/clap/builder_derive_interop/Cargo.lock) · [src/main.rs](examples/clap/builder_derive_interop/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-builder_derive_interop"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{Args, Command, FromArgMatches, Subcommand};

#[derive(Debug, Args)]
struct Common {
    #[arg(long)]
    debug: bool,
}

#[derive(Debug, Subcommand)]
enum Actions {
    /// Greet someone.
    Greet { name: String },
}

fn cli() -> Command {
    let command = Command::new("app").subcommand_required(true);
    let command = Common::augment_args(command);
    Actions::augment_subcommands(command)
}

fn main() -> Result<(), clap::Error> {
    cli().debug_assert();
    let matches = cli().try_get_matches_from(["app", "--debug", "greet", "Ada"])?;
    let common = Common::from_arg_matches(&matches)?;
    let action = Actions::from_arg_matches(&matches)?;
    assert!(common.debug);
    let Actions::Greet { name } = action;
    assert_eq!(name, "Ada");
    println!("Hello {name}!");
    Ok(())
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/clap/builder_derive_interop/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/clap/builder_derive_interop/Cargo.toml`. This is a finite executable.

Release evidence: [documentation, lines 473–485](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [documentation, lines 457–464](https://docs.rs/crate/clap/4.6.7/source/src/_derive/mod.rs) · [api, lines 127–165](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L127-L165) · [api, lines 227–245](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L227-L245) · [api, lines 262–278](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L262-L278) · [test, lines 50–59](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/tests/builder/app_settings.rs#L50-L59).

### Capture positional values after --

Capture an escaped positional tail while preserving a hyphen-prefixed value and an already-tokenized value containing spaces.

[Cargo.toml](examples/clap/escaped_positionals/Cargo.toml) · [Cargo.lock](examples/clap/escaped_positionals/Cargo.lock) · [src/main.rs](examples/clap/escaped_positionals/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-escaped_positionals"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{CommandFactory, Parser};

#[derive(Debug, Parser)]
#[command(name = "wrapper")]
struct Cli {
    #[arg(short = 'f')]
    flag: bool,
    #[arg(short = 'p', value_name = "PEAR")]
    pear: Option<String>,
    /// Values accepted only after --.
    #[arg(last = true)]
    rest: Vec<String>,
}

fn main() -> Result<(), clap::Error> {
    Cli::command().debug_assert();
    let cli = Cli::try_parse_from([
        "wrapper", "-f", "-p=bob", "--", "--child-flag", "two words",
    ])?;
    assert!(cli.flag);
    assert_eq!(cli.pear.as_deref(), Some("bob"));
    assert_eq!(cli.rest, vec!["--child-flag".to_owned(), "two words".to_owned()]);
    assert!(Cli::try_parse_from(["wrapper", "unescaped"]).is_err());
    println!("{:?}", cli.rest);
    Ok(())
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/clap/escaped_positionals/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/clap/escaped_positionals/Cargo.toml`. This is a finite executable.

Release evidence: [example, lines 1–25](https://docs.rs/crate/clap/4.6.7/source/examples/escaped-positional-derive.rs) · [api, lines 59–73](https://docs.rs/crate/clap/4.6.7/source/src/_concepts.rs) · [api, lines 70–78](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L70-L78).

### Accept an option with an optional enum value

Distinguish an omitted option, a bare switch-like option, and an explicit enum value. Explicit values must use the equals form.

[Cargo.toml](examples/clap/optional_value/Cargo.toml) · [Cargo.lock](examples/clap/optional_value/Cargo.lock) · [src/main.rs](examples/clap/optional_value/src/main.rs)

```toml
[dependencies.clap]
default-features = true
features = ["derive"]
version = "=4.6.7"

[package]
edition = "2024"
name = "recipe-optional_value"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use clap::{CommandFactory, Parser, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ColorWhen {
    Always,
    Auto,
    Never,
}

#[derive(Debug, Parser)]
#[command(name = "viewer")]
struct Cli {
    #[arg(
        long,
        value_enum,
        num_args = 0..=1,
        require_equals = true,
        default_value_t = ColorWhen::Auto,
        default_missing_value = "always"
    )]
    color: ColorWhen,
}

fn main() -> Result<(), clap::Error> {
    Cli::command().debug_assert();
    assert_eq!(Cli::try_parse_from(["viewer"])?.color, ColorWhen::Auto);
    assert_eq!(Cli::try_parse_from(["viewer", "--color"])?.color, ColorWhen::Always);
    assert_eq!(Cli::try_parse_from(["viewer", "--color=never"])?.color, ColorWhen::Never);
    assert!(Cli::try_parse_from(["viewer", "--color=unknown"]).is_err());
    assert!(Cli::try_parse_from(["viewer", "--color", "never"]).is_err());
    Ok(())
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/clap/optional_value/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/clap/optional_value/Cargo.toml`. This is a finite executable.

Release evidence: [example, lines 33–42](https://docs.rs/crate/clap/4.6.7/source/examples/git-derive.rs) · [example, lines 62–67](https://docs.rs/crate/clap/4.6.7/source/examples/git-derive.rs) · [example, lines 25–31](https://docs.rs/crate/clap/4.6.7/source/examples/git.rs) · [api, lines 281–286](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L281-L286) · [api, lines 70–78](https://github.com/clap-rs/clap/blob/13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab/clap_builder/src/derive.rs#L70-L78).

## axum 0.8.9

Symposiast generated this guidance through its Codex adapter from frozen release evidence. The bundle contains 49 sources and 13 recipes. Its 78 inventoried documentation examples have 30 adapted, 27 supporting, and 21 skipped decisions. Inspect the [complete inventory and coverage reasons](examples/axum/coverage.json). Standalone upstream example programs are also evidence, but are not counted as documentation code blocks. All recipes passed compilation after analysis; the agent's original pre-verification summary is preserved in the bundle.

Install the [checked-in bundle](https://github.com/mcaveniathor/symposiast/tree/main/plugins/axum) with `symposiast install plugins/axum/bundle`, then run `cargo agents sync` in the consumer workspace. Skills and MCP retrieval target 0.8.9 exactly. Restart the consuming MCP session after installation.

### Findings used by the generated recipes

- **Select server features and runtime dependencies explicitly** (documented): Use Rust 1.80 or newer for axum 0.8.9. With default features disabled, serve requires tokio and either http1 or http2. The default #\[tokio::main\] runtime also needs the application's Tokio macros and rt-multi-thread features. [manifest, lines 12–16](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [api, lines 44–45](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [api, lines 528–530](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).
- **Update historical route syntax before reuse** (documented): Use /\{id\} and /\{\*rest\} captures. Historical /:id and /\*rest patterns panic under the normal 0.8 checks. Wildcards do not cover the bare prefix or its trailing slash; register those paths separately when needed. Automatic trailing-slash redirects are absent. [changelog, lines 121–122](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 601–625](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 661–680](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md).
- **Separate nested-router and method fallback behavior** (documented): A router nested with nest inherits the outer fallback unless it supplies its own. nest\_service does not inherit fallbacks. MethodRouter fallbacks handle unmatched methods, and merging two method routers that both have custom fallbacks panics. Set one fallback after merging. [changelog, lines 573–599](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 352–353](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [documentation, lines 24–55](https://docs.rs/crate/axum/0.8.9/source/src/docs/method_routing/fallback.md).
- **Use typed state and inexpensive clones** (documented): Prefer State for application-wide data. Router&lt;S&gt; represents a router still missing S; supply it with with\_state before serving. Keep state clones cheap, using Arc where appropriate. Implement FromRef for substates, or enable macros to derive it. A missing required Extension causes a runtime 500. [api, lines 186–232](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [api, lines 267–270](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).
- **Order extractors around one body consumer** (documented): Extractors run left to right. Put Path, Query, headers, and State before the single body-consuming extractor. Json, Form, String, Bytes, and Request occupy the final position when used. JSON needs a suitable content type; Form reads GET/HEAD query strings and otherwise expects an application/x-www-form-urlencoded body. [documentation, lines 120–193](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [api, lines 14–24](https://docs.rs/crate/axum/0.8.9/source/src/json.rs) · [api, lines 13–28](https://docs.rs/crate/axum/0.8.9/source/src/form.rs).
- **Customize rejections through their public interface** (documented): Use Result&lt;Json&lt;T&gt;, JsonRejection&gt; for handler-local error customization, or wrap Json in a custom extractor for reuse. Public status() and body\_text() methods support an error envelope without inspecting internal error types. Internal-source downcasts can break without a major release. [documentation, lines 266–277](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [documentation, lines 389–391](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [example, lines 45–55](https://github.com/tokio-rs/axum/blob/c59208c86fded335cd85e388030ad59347b0e5ae/examples/customize-extractor-error/src/custom_extractor.rs#L45-L55).
- **Check each optional extractor's actual semantics** (inferred): Do not use Option&lt;Extractor&gt; as a general error-suppression mechanism. In this release, Option&lt;Json&lt;T&gt;&gt; returns None when Content-Type is absent but still rejects an unsuitable content type or invalid JSON. Option&lt;Extension&lt;T&gt;&gt; returns None for a missing extension. [documentation, lines 238–247](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [api, lines 116–135](https://docs.rs/crate/axum/0.8.9/source/src/json.rs) · [api, lines 100–112](https://docs.rs/crate/axum/0.8.9/source/src/extension.rs).
- **Choose the appropriate custom extractor trait** (documented): Implement FromRequestParts&lt;S&gt; for metadata and FromRequest&lt;S&gt; for body-consuming extraction, using native async methods. Avoid implementing both directly on one concrete extractor; generic wrappers are the documented exception. Historical request-body-generic signatures should not be copied. [documentation, lines 393–483](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [documentation, lines 485–542](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [documentation, lines 607–611](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [changelog, lines 52–55](https://github.com/tokio-rs/axum/blob/c59208c86fded335cd85e388030ad59347b0e5ae/axum-core/CHANGELOG.md#L52-L55).
- **Keep body buffering limits explicit** (documented): Bytes-based extractors have a default 2 MB limit, also affecting String, Json, and Form. When collecting Body directly with to\_bytes, supply the desired limit explicitly. Its error source can identify http\_body\_util::LengthLimitError. [documentation, lines 599–605](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [api, lines 14–54](https://docs.rs/crate/axum/0.8.9/source/src/body/mod.rs).
- **Convert service failures into responses** (documented): Handler Result errors that implement IntoResponse become ordinary HTTP responses. Fallible Tower services and middleware require HandleError or HandleErrorLayer. Place HandleErrorLayer outside a Tower timeout; any request-parts extractors precede the error argument. [documentation, lines 5–36](https://docs.rs/crate/axum/0.8.9/source/src/docs/error_handling.md) · [documentation, lines 49–92](https://docs.rs/crate/axum/0.8.9/source/src/docs/error_handling.md) · [documentation, lines 94–174](https://docs.rs/crate/axum/0.8.9/source/src/docs/error_handling.md).
- **Account for middleware scope and ordering** (documented): Add routes before applying their layers. Repeated layer calls process incoming requests from the last layer backward; ServiceBuilder processes them in declaration order. URI rewriting must wrap the entire Router because Router::layer runs after routing. [documentation, lines 20–23](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 63–65](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 145–152](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 525–532](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 6–8](https://docs.rs/crate/axum/0.8.9/source/src/docs/method_routing/layer.md).
- **Keep request context and backpressure policies deliberate** (documented): Pass per-request middleware data through request extensions and insert it before next.run. Request extensions are not automatically copied to the response. Route-level concurrency middleware does not establish a general backpressure policy; the documentation recommends load shedding or placing backpressure-sensitive middleware around the whole application. [documentation, lines 343–383](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 465–523](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md).
- **Make response and error types inferable** (documented): Use tuples to compose status, headers, extensions, and a final body. Convert heterogeneous branches into Response with into\_response(). For question-mark propagation, give Result a concrete error type; impl IntoResponse alone does not resolve error conversion or allow unrelated branch types. [documentation, lines 150–185](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [documentation, lines 240–317](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md).
- **Diagnose handler bounds and extractor failures** (documented): For Handler errors, check extractor order, response conversion, Send futures, and Sync requirements. Holding a non-Send value across await commonly breaks a handler. Enable macros for debug\_handler diagnostics. Enable tracing and configure the axum::rejection=trace target to inspect built-in extractor rejections. [documentation, lines 3–36](https://docs.rs/crate/axum/0.8.9/source/src/docs/debugging_handler_type_errors.md) · [changelog, lines 123–124](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [api, lines 525–526](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [documentation, lines 676–681](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md).

### Start a typed JSON HTTP server

Serve a greeting and accept POST /users with a JSON username, returning an illustrative 201 JSON response. Runs until stopped.

[Cargo.toml](examples/axum/r01/Cargo.toml) · [Cargo.lock](examples/axum/r01/Cargo.lock) · [src/main.rs](examples/axum/r01/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
    "json",
]
version = "=0.8.9"

[dependencies.serde]
default-features = true
features = ["derive"]
version = "1.0.221"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r01"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{http::StatusCode, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

async fn create_user(Json(input): Json<CreateUser>) -> (StatusCode, Json<User>) {
    // Demonstration response; no database write occurs.
    (StatusCode::CREATED, Json(User { id: 1337, username: input.username }))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/users", post(create_user));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r01/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r01/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [readme, lines 28–86](https://docs.rs/crate/axum/0.8.9/source/README.md) · [api, lines 27–45](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [api, lines 103–127](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [documentation, lines 12–33](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [api, lines 33–53](https://docs.rs/crate/axum/0.8.9/source/src/json.rs) · [manifest, lines 339–341](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml).

### Compose routes and fallbacks without historical API traps

Demonstrate 0.8 wildcard syntax, explicit slash routes, nested fallback inheritance, and a shared method fallback. Runs until stopped.

[Cargo.toml](examples/axum/r02/Cargo.toml) · [Cargo.lock](examples/axum/r02/Cargo.lock) · [src/main.rs](examples/axum/r02/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
]
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r02"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{extract::Path, http::{Method, StatusCode, Uri}, routing::{get, post}, Router};

async fn wildcard(Path(rest): Path<String>) -> String {
    rest
}

async fn outer_fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "outer fallback")
}

async fn method_fallback(method: Method, uri: Uri) -> (StatusCode, String) {
    (StatusCode::METHOD_NOT_ALLOWED, format!("{method} is not allowed for {uri}"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let api = Router::new()
        .route("/users", get(|| async { "users" }))
        .fallback(|| async { (StatusCode::NOT_FOUND, "API fallback") });
    let shared = Router::new().route("/users", get(|| async { "shared users" }));
    let methods = get(|| async { "GET" })
        .merge(post(|| async { "POST" }))
        .fallback(method_fallback);
    let app = Router::new()
        .nest("/api", api)
        .nest("/shared", shared)
        .route("/foo/{*rest}", get(wildcard))
        .route("/foo/bar", get(|| async { "static route" }))
        .route("/foo", get(|| async { "bare prefix" }))
        .route("/foo/", get(|| async { "trailing slash" }))
        .route("/methods", methods)
        .fallback(outer_fallback);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r02/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r02/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [changelog, lines 121–122](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 575–599](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 606–621](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 629–641](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 648–659](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 665–677](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [changelog, lines 699–705](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [api, lines 51–66](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [documentation, lines 5–46](https://docs.rs/crate/axum/0.8.9/source/src/docs/method_routing/fallback.md) · [documentation, lines 6–23](https://docs.rs/crate/axum/0.8.9/source/src/docs/method_routing/merge.md) · [test, lines 84–110](https://docs.rs/crate/axum/0.8.9/source/src/routing/tests/fallback.rs) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Share application state and extract a substate

Build routes that require AppState, provide that state before serving, and extract only an Arc-backed substate. No macros feature is needed for manual FromRef.

[Cargo.toml](examples/axum/r03/Cargo.toml) · [Cargo.lock](examples/axum/r03/Cargo.lock) · [src/main.rs](examples/axum/r03/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
]
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r03"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{extract::{FromRef, State}, routing::get, Router};
use std::sync::Arc;

struct Settings {
    greeting: String,
}

#[derive(Clone)]
struct AppState {
    settings: Arc<Settings>,
}

impl FromRef<AppState> for Arc<Settings> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.settings)
    }
}

async fn greeting(State(settings): State<Arc<Settings>>) -> String {
    settings.greeting.clone()
}

fn routes() -> Router<AppState> {
    Router::new().route("/", get(greeting))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = AppState {
        settings: Arc::new(Settings { greeting: "Hello from shared state".to_owned() }),
    };
    let app: Router = routes().with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r03/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r03/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [changelog, lines 753–764](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [api, lines 160–232](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Combine typed path and query parameters

Deserialize a UUID path and page/per\_page query parameters, then return typed JSON. UUID serialization support is enabled without requesting UUID generation.

[Cargo.toml](examples/axum/r04/Cargo.toml) · [Cargo.lock](examples/axum/r04/Cargo.lock) · [src/main.rs](examples/axum/r04/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
    "query",
    "json",
]
version = "=0.8.9"

[dependencies.serde]
default-features = true
features = ["derive"]
version = "1.0.221"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[dependencies.uuid]
default-features = true
features = ["serde"]
version = "1.0"

[package]
edition = "2024"
name = "recipe-r04"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{extract::{Path, Query}, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
struct Pagination {
    page: usize,
    per_page: usize,
}

#[derive(Serialize)]
struct User {
    id: Uuid,
    username: String,
}

#[derive(Serialize)]
struct Page {
    user_id: Uuid,
    page: usize,
    per_page: usize,
}

async fn user(Path(id): Path<Uuid>) -> Json<User> {
    // Illustrative lookup result.
    Json(User { id, username: "example".to_owned() })
}

async fn things(Path(user_id): Path<Uuid>, Query(p): Query<Pagination>) -> Json<Page> {
    Json(Page { user_id, page: p.page, per_page: p.per_page })
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new()
        .route("/users/{id}", get(user))
        .route("/users/{id}/things", get(things));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r04/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r04/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [api, lines 81–95](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs) · [documentation, lines 94–118](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [api, lines 65–93](https://docs.rs/crate/axum/0.8.9/source/src/json.rs) · [manifest, lines 419–424](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Return a consistent JSON rejection envelope

Echo valid typed JSON and turn every JSON rejection into a JSON message while preserving its status. The body extractor remains last.

[Cargo.toml](examples/axum/r05/Cargo.toml) · [Cargo.lock](examples/axum/r05/Cargo.lock) · [src/main.rs](examples/axum/r05/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
    "json",
]
version = "=0.8.9"

[dependencies.serde]
default-features = true
features = ["derive"]
version = "1.0.221"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r05"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{extract::rejection::JsonRejection, http::{HeaderMap, StatusCode}, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Payload {
    username: String,
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

async fn echo(
    _headers: HeaderMap,
    payload: Result<Json<Payload>, JsonRejection>,
) -> Result<Json<Payload>, (StatusCode, Json<ErrorBody>)> {
    payload.map_err(|rejection| {
        let status = rejection.status();
        let body = ErrorBody { message: rejection.body_text() };
        (status, Json(body))
    })
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new().route("/echo", post(echo));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r05/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r05/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [changelog, lines 807–828](https://docs.rs/crate/axum/0.8.9/source/CHANGELOG.md) · [documentation, lines 132–190](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [documentation, lines 201–236](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [example, lines 45–55](https://github.com/tokio-rs/axum/blob/c59208c86fded335cd85e388030ad59347b0e5ae/examples/customize-extractor-error/src/custom_extractor.rs#L45-L55) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Implement metadata and body validation extractors

Require a User-Agent header and a nonempty body using separate extractor traits. Delegating to Bytes retains its standard rejection and body-limit behavior.

[Cargo.toml](examples/axum/r06/Cargo.toml) · [Cargo.lock](examples/axum/r06/Cargo.lock) · [src/main.rs](examples/axum/r06/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
]
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r06"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{
    body::Bytes,
    extract::{FromRequest, FromRequestParts, Request},
    http::{header::{HeaderValue, USER_AGENT}, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};

struct UserAgent(HeaderValue);

impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.headers.get(USER_AGENT).cloned().map(Self)
            .ok_or((StatusCode::BAD_REQUEST, "User-Agent is required"))
    }
}

struct NonemptyBody(Bytes);

impl<S> FromRequest<S> for NonemptyBody
where
    S: Send + Sync,
    Bytes: FromRequest<S>,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state).await
            .map_err(IntoResponse::into_response)?;
        if bytes.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Body must not be empty").into_response());
        }
        Ok(Self(bytes))
    }
}

async fn echo(UserAgent(agent): UserAgent, NonemptyBody(body): NonemptyBody) -> impl IntoResponse {
    ([("x-client-agent", agent)], body)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new().route("/echo", post(echo));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r06/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r06/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [documentation, lines 403–438](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [documentation, lines 444–483](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [documentation, lines 491–542](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [documentation, lines 599–605](https://docs.rs/crate/axum/0.8.9/source/src/docs/extract.md) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Pass request context from middleware to handlers

Demonstrate middleware inserting per-request identity for a handler. The fixed token is illustrative; unmatched paths retain the router's 404 behavior.

[Cargo.toml](examples/axum/r07/Cargo.toml) · [Cargo.lock](examples/axum/r07/Cargo.lock) · [src/main.rs](examples/axum/r07/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
]
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r07"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{
    extract::{Extension, Request},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};

#[derive(Clone)]
struct CurrentUser {
    name: String,
}

async fn auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Demonstration only: replace this fixed token check with real authentication.
    let authorized = req.headers().get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        == Some("Bearer demo-token");
    if !authorized {
        return Err(StatusCode::UNAUTHORIZED);
    }
    req.extensions_mut().insert(CurrentUser { name: "demo".to_owned() });
    Ok(next.run(req).await)
}

async fn profile(Extension(user): Extension<CurrentUser>) -> String {
    format!("Hello, {}", user.name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new()
        .route("/profile", get(profile))
        .route_layer(middleware::from_fn(auth));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r07/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r07/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [documentation, lines 469–519](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [api, lines 46–48](https://docs.rs/crate/axum/0.8.9/source/src/extension.rs) · [test, lines 487–512](https://docs.rs/crate/axum/0.8.9/source/src/routing/tests/mod.rs) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Handle fallible services and Tower timeouts

Serve a deliberately failing service as HTTP 500 and a deliberately pending handler as HTTP 408 after one second, with error conversion outside the timeout.

[Cargo.toml](examples/axum/r08/Cargo.toml) · [Cargo.lock](examples/axum/r08/Cargo.lock) · [src/main.rs](examples/axum/r08/src/main.rs)

```toml
[dependencies.anyhow]
default-features = true
features = []
version = "1.0"

[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
]
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
    "time",
]
version = "1.44.2"

[dependencies.tower]
default-features = false
features = [
    "util",
    "timeout",
]
version = "0.5.2"

[package]
edition = "2024"
name = "recipe-r08"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{
    error_handling::{HandleError, HandleErrorLayer},
    extract::Request,
    http::{Method, StatusCode, Uri},
    response::Response,
    routing::get,
    BoxError, Router,
};
use std::time::Duration;
use tower::ServiceBuilder;

async fn slow() -> &'static str {
    std::future::pending::<()>().await;
    "finished"
}

async fn handle_error(method: Method, uri: Uri, error: BoxError) -> (StatusCode, String) {
    let status = if error.is::<tower::timeout::error::Elapsed>() {
        StatusCode::REQUEST_TIMEOUT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    (status, format!("{method} {uri} could not be completed"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let fallible = tower::service_fn(|_req: Request| async {
        Err::<Response, anyhow::Error>(anyhow::anyhow!("demonstration failure"))
    });
    let app = Router::new()
        .route_service("/service", HandleError::new(fallible, |_error: anyhow::Error| async {
            (StatusCode::INTERNAL_SERVER_ERROR, "Service failed")
        }))
        .route("/slow", get(slow))
        .layer(ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_error))
            .timeout(Duration::from_secs(1)));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r08/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r08/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [documentation, lines 56–92](https://docs.rs/crate/axum/0.8.9/source/src/docs/error_handling.md) · [documentation, lines 99–134](https://docs.rs/crate/axum/0.8.9/source/src/docs/error_handling.md) · [documentation, lines 140–174](https://docs.rs/crate/axum/0.8.9/source/src/docs/error_handling.md) · [documentation, lines 259–268](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 296–321](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [manifest, lines 368–378](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [manifest, lines 317–318](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Compose responses and resolve return-type inference

Show tuple composition, response extensions, a low-level builder, heterogeneous response branches, and a concrete Result error type. POST /create only demonstrates header validation.

[Cargo.toml](examples/axum/r09/Cargo.toml) · [Cargo.lock](examples/axum/r09/Cargo.lock) · [src/main.rs](examples/axum/r09/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
]
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r09"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{
    body::Body,
    http::{HeaderMap, StatusCode, Uri},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Extension, Router,
};

#[derive(Clone)]
struct ResponseTag;

async fn composed() -> impl IntoResponse {
    (StatusCode::OK, [("x-example", "tuple")], Extension(ResponseTag), "Hello")
}

async fn raw() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("x-example", "builder")
        .body(Body::from("not found"))
        .unwrap()
}

async fn choice(uri: Uri) -> Response {
    match uri.query() {
        Some("format=html") => Html("<p>Hello</p>").into_response(),
        Some("redirect=true") => Redirect::to("/").into_response(),
        _ => "Hello".into_response(),
    }
}

fn validate(headers: &HeaderMap) -> Result<(), StatusCode> {
    if headers.contains_key("x-name") {
        Ok(())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn create(headers: HeaderMap) -> Result<StatusCode, StatusCode> {
    validate(&headers)?;
    Ok(StatusCode::CREATED)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new()
        .route("/", get(composed))
        .route("/raw", get(raw))
        .route("/choice", get(choice))
        .route("/create", post(create));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r09/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r09/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [documentation, lines 16–28](https://docs.rs/crate/axum/0.8.9/source/src/docs/error_handling.md) · [api, lines 54–69](https://docs.rs/crate/axum/0.8.9/source/src/extension.rs) · [documentation, lines 8–66](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [documentation, lines 71–148](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [documentation, lines 165–180](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [documentation, lines 187–215](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [documentation, lines 222–238](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [documentation, lines 243–258](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [documentation, lines 266–278](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [documentation, lines 286–314](https://docs.rs/crate/axum/0.8.9/source/src/docs/response.md) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Read and return URL-encoded forms

Accept GET /form?value=hello or a POST form body value=hello with application/x-www-form-urlencoded, and return an encoded form response.

[Cargo.toml](examples/axum/r10/Cargo.toml) · [Cargo.lock](examples/axum/r10/Cargo.lock) · [src/main.rs](examples/axum/r10/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
    "form",
]
version = "=0.8.9"

[dependencies.serde]
default-features = true
features = ["derive"]
version = "1.0.221"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r10"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{routing::get, Form, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Payload {
    value: String,
}

async fn echo(Form(payload): Form<Payload>) -> Form<Payload> {
    Form(payload)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new().route("/form", get(echo).post(echo));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r10/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r10/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [api, lines 13–64](https://docs.rs/crate/axum/0.8.9/source/src/form.rs) · [manifest, lines 92–96](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

### Collect a body with an explicit byte limit

A finite program illustrating successful bounded collection and recognition of a size-limit error. It needs no axum server features.

[Cargo.toml](examples/axum/r11/Cargo.toml) · [Cargo.lock](examples/axum/r11/Cargo.lock) · [src/main.rs](examples/axum/r11/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = []
version = "=0.8.9"

[dependencies.http-body-util]
default-features = true
features = []
version = "0.1.0"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt",
]
version = "1.44.2"

[package]
edition = "2024"
name = "recipe-r11"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::body::{to_bytes, Body};
use http_body_util::LengthLimitError;
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), axum::Error> {
    let bytes = to_bytes(Body::from(vec![1_u8, 2, 3]), 3).await?;
    assert_eq!(&bytes[..], &[1, 2, 3]);

    let error = to_bytes(Body::from(vec![1_u8, 2, 3]), 1)
        .await
        .expect_err("three bytes exceed a one-byte limit");
    assert!(error.source().is_some_and(|source| source.is::<LengthLimitError>()));
    Ok(())
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r11/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r11/Cargo.toml`. This is a finite executable.

Release evidence: [api, lines 18–47](https://docs.rs/crate/axum/0.8.9/source/src/body/mod.rs) · [manifest, lines 183–184](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [manifest, lines 351–359](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml).

### Rewrite a URI before routing

Internally rewrite /legacy to / before route selection, preserving the query string. The layer wraps the whole Router.

[Cargo.toml](examples/axum/r12/Cargo.toml) · [Cargo.lock](examples/axum/r12/Cargo.lock) · [src/main.rs](examples/axum/r12/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
]
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[dependencies.tower]
default-features = false
features = ["util"]
version = "0.5.2"

[package]
edition = "2024"
name = "recipe-r12"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{extract::Request, routing::get, Router, ServiceExt};
use tower::{util::MapRequestLayer, Layer};

fn rewrite(mut request: Request) -> Request {
    if request.uri().path() == "/legacy" {
        let target = match request.uri().query() {
            Some(query) => format!("/?{query}"),
            None => "/".to_owned(),
        };
        *request.uri_mut() = target.parse().expect("valid path with the existing query");
    }
    request
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app: Router = Router::new().route("/", get(|| async { "Current endpoint" }));
    let wrapped = MapRequestLayer::new(rewrite).layer(app);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, wrapped.into_make_service()).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r12/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r12/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [documentation, lines 525–562](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [api, lines 7–18](https://docs.rs/crate/axum/0.8.9/source/src/service_ext.rs) · [manifest, lines 268–271](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml).

### Compose tracing, extensions, and a method concurrency limit

Trace HTTP requests, insert an extension before the handler, and limit concurrent calls through the GET method route. This demonstrates layer composition, not a complete overload policy.

[Cargo.toml](examples/axum/r13/Cargo.toml) · [Cargo.lock](examples/axum/r13/Cargo.lock) · [src/main.rs](examples/axum/r13/src/main.rs)

```toml
[dependencies.axum]
default-features = false
features = [
    "tokio",
    "http1",
    "tracing",
]
version = "=0.8.9"

[dependencies.tokio]
default-features = true
features = [
    "macros",
    "rt-multi-thread",
    "net",
]
version = "1.44.2"

[dependencies.tower]
default-features = false
features = [
    "util",
    "limit",
]
version = "0.5.2"

[dependencies.tower-http]
default-features = false
features = ["trace"]
version = "0.6.8"

[dependencies.tracing-subscriber]
default-features = true
features = []
version = "0.3"

[package]
edition = "2024"
name = "recipe-r13"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use axum::{extract::Extension, routing::get, Router};
use tower::{limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppTag(&'static str);

async fn handler(Extension(tag): Extension<AppTag>) -> String {
    format!("Application: {}", tag.0)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();
    let app = Router::new()
        .route("/", get(handler).layer(ConcurrencyLimitLayer::new(64)))
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(Extension(AppTag("example"))));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/axum/r13/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/axum/r13/Cargo.toml`. This starts a server at `http://127.0.0.1:3000`; stop it with Ctrl-C before running another server recipe.

Release evidence: [documentation, lines 25–47](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 69–84](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 124–152](https://docs.rs/crate/axum/0.8.9/source/src/docs/middleware.md) · [documentation, lines 15–27](https://docs.rs/crate/axum/0.8.9/source/src/docs/method_routing/layer.md) · [manifest, lines 273–304](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [manifest, lines 368–378](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [manifest, lines 415–417](https://docs.rs/crate/axum/0.8.9/source/Cargo.toml) · [api, lines 27–42](https://docs.rs/crate/axum/0.8.9/source/src/lib.rs).

## Bevy 0.20.0

Symposiast generated this guidance through its Codex adapter from frozen release evidence. The bundle contains 47 sources and 5 recipes. Its 5 inventoried documentation examples have 3 adapted, 0 supporting, and 2 skipped decisions. Inspect the [complete inventory and coverage reasons](examples/bevy/coverage.json). Standalone upstream example programs are also evidence, but are not counted as documentation code blocks. All recipes passed compilation after analysis; the agent's original pre-verification summary is preserved in the bundle.

The latest stable, non-yanked release resolved on October 21, 2026 was **0.20.0**. Its declared minimum Rust version is 1.97.1. The pinned repository archive exceeds the collector's size limit, so this bundle uses the verified published crate. The manifests are truncated and source selection is bounded; this is a focused sample, not full engine or migration coverage.

Install the [checked-in bundle](https://github.com/mcaveniathor/symposiast/tree/main/plugins/bevy) with `symposiast install plugins/bevy/bundle`, then run `cargo agents sync` in the consumer workspace. Skills and MCP retrieval target 0.20.0 exactly. Restart the consuming MCP session after installation.

### Findings used by the generated recipes

- **Choose the release toolchain and feature scope explicitly** (documented): Use Rust 1.97.1 or newer for Bevy 0.20.0. Disable default features when selecting a domain profile such as 2d; defaults combine 2d, 3d, ui, and audio. For applications without rendering, default\_app is a documented baseline. [manifest, lines 12–16](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml) · [documentation, lines 7–14](https://docs.rs/crate/bevy/0.20.0/source/docs/cargo_features.md) · [manifest, lines 135–142](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml.orig) · [manifest, lines 175–182](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml.orig).
- **Separate release evidence from illustrative version labels** (inferred): Keep dependency versions pinned to 0.20.0 when adapting this evidence. The feature documentation's TOML example still names 0.18, and the migration-process examples use illustrative older versions and placeholder methods; neither establishes a 0.20 API migration. [documentation, lines 16–20](https://docs.rs/crate/bevy/0.20.0/source/docs/cargo_features.md) · [manifest, lines 12–16](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml) · [changelog, lines 75–88](https://docs.rs/crate/bevy/0.20.0/source/_release-content/migration_guides.md).
- **Register systems and select plugins separately** (documented): Bootstrap standard window functionality with App::new().add\_plugins(DefaultPlugins).run(). For the basic console example, register a function with add\_systems(Update, system) and run the app without adding the window plugin group. [readme, lines 72–82](https://docs.rs/crate/bevy/0.20.0/source/README.md) · [api, lines 17–30](https://docs.rs/crate/bevy/0.20.0/source/src/lib.rs).
- **Drive headless applications with explicit updates** (documented): For headless application checks, use MinimalPlugins and call app.update() explicitly. Supply resources omitted by those plugins, such as ButtonInput&lt;KeyCode&gt;, yourself. When manually simulating input without InputPlugin, clear just\_pressed state between updates. Splitting application logic into plugins supports isolated checks. [test, lines 1–9](https://docs.rs/crate/bevy/0.20.0/source/tests/how_to_test_apps.rs) · [test, lines 23–24](https://docs.rs/crate/bevy/0.20.0/source/tests/how_to_test_apps.rs) · [test, lines 52–85](https://docs.rs/crate/bevy/0.20.0/source/tests/how_to_test_apps.rs) · [test, lines 88–128](https://docs.rs/crate/bevy/0.20.0/source/tests/how_to_test_apps.rs).
- **Register buffered messages before writing or reading them** (inferred): For buffered ECS communication, derive Message, register the type with add\_message, and consume it through MessageReader::read. The supplied test demonstrates injecting a message through Messages&lt;T&gt;, advancing the application once, and inspecting the resulting resource. [test, lines 10–20](https://docs.rs/crate/bevy/0.20.0/source/tests/how_to_test_systems.rs) · [test, lines 150–174](https://docs.rs/crate/bevy/0.20.0/source/tests/how_to_test_systems.rs).
- **Wrap mesh and material handles in rendering components** (documented): Create primitive meshes through Assets&lt;Mesh&gt; and materials through Assets&lt;ColorMaterial&gt;. Wrap their handles in Mesh2d and MeshMaterial2d when spawning a renderable entity, and spawn Camera2d to view it. [example, lines 4–10](https://docs.rs/crate/bevy/0.20.0/source/examples/2d/2d_shapes.rs) · [example, lines 12–24](https://docs.rs/crate/bevy/0.20.0/source/examples/2d/mesh2d.rs).
- **Keep span guards alive while profiling work** (documented): Select a tracing backend, such as trace\_chrome, which also enables trace and debug. Keep info-level spans enabled. Bind the guard returned by info\_span!(...).entered() for the full synchronous work scope; use a separate enter() guard only when finer control is needed. Profile release builds; Chrome-format output can be inspected with Perfetto. [documentation, lines 24–28](https://docs.rs/crate/bevy/0.20.0/source/docs/profiling.md) · [documentation, lines 39–59](https://docs.rs/crate/bevy/0.20.0/source/docs/profiling.md) · [documentation, lines 131–135](https://docs.rs/crate/bevy/0.20.0/source/docs/profiling.md) · [manifest, lines 443–453](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml.orig).

### Open a window with default plugins

Open the documented standard window using the full default feature set. Running this graphical recipe requires a suitable display and graphics environment.

[Cargo.toml](examples/bevy/window/Cargo.toml) · [Cargo.lock](examples/bevy/window/Cargo.lock) · [src/main.rs](examples/bevy/window/src/main.rs)

```toml
[dependencies.bevy]
default-features = true
features = []
version = "=0.20.0"

[package]
edition = "2024"
name = "recipe-window"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .run();
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/bevy/window/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/bevy/window/Cargo.toml`. This requires a display and graphics environment. Compilation was verified; graphical runtime behavior was not checked.

Release evidence: [readme, lines 74–82](https://docs.rs/crate/bevy/0.20.0/source/README.md) · [manifest, lines 135–136](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml.orig).

### Run a console system

Use the crate's hello-world program with the documented default\_app baseline for applications without rendering. No additional plugins are installed.

[Cargo.toml](examples/bevy/hello/Cargo.toml) · [Cargo.lock](examples/bevy/hello/Cargo.lock) · [src/main.rs](examples/bevy/hello/src/main.rs)

```toml
[dependencies.bevy]
default-features = false
features = ["default_app"]
version = "=0.20.0"

[package]
edition = "2024"
name = "recipe-hello"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Update, hello_world_system)
        .run();
}

fn hello_world_system() {
    println!("hello world");
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/bevy/hello/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/bevy/hello/Cargo.toml`. This is a finite executable.

Release evidence: [api, lines 18–30](https://docs.rs/crate/bevy/0.20.0/source/src/lib.rs) · [manifest, lines 175–182](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml.orig).

### Process a message in a headless application

Adapt the supplied system test into a standalone program that registers a message, injects it, advances the app, and asserts the resulting score.

[Cargo.toml](examples/bevy/message/Cargo.toml) · [Cargo.lock](examples/bevy/message/Cargo.lock) · [src/main.rs](examples/bevy/message/src/main.rs)

```toml
[dependencies.bevy]
default-features = false
features = ["default_app"]
version = "=0.20.0"

[package]
edition = "2024"
name = "recipe-message"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use bevy::prelude::*;

#[derive(Message)]
struct EnemyDied(u32);

#[derive(Resource)]
struct Score(u32);

fn update_score(mut messages: MessageReader<EnemyDied>, mut score: ResMut<Score>) {
    for message in messages.read() {
        score.0 += message.0;
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(Score(0))
        .add_message::<EnemyDied>()
        .add_systems(Update, update_score);

    app.world_mut()
        .resource_mut::<Messages<EnemyDied>>()
        .write(EnemyDied(3));

    app.update();

    assert_eq!(app.world().resource::<Score>().0, 3);
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/bevy/message/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/bevy/message/Cargo.toml`. This is a finite executable.

Release evidence: [test, lines 10–20](https://docs.rs/crate/bevy/0.20.0/source/tests/how_to_test_systems.rs) · [test, lines 150–174](https://docs.rs/crate/bevy/0.20.0/source/tests/how_to_test_systems.rs) · [manifest, lines 175–182](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml.orig).

### Render a primitive 2D mesh

Render a purple rectangle using the 2d profile and generated assets. No external asset files are needed.

[Cargo.toml](examples/bevy/mesh/Cargo.toml) · [Cargo.lock](examples/bevy/mesh/Cargo.lock) · [src/main.rs](examples/bevy/mesh/src/main.rs)

```toml
[dependencies.bevy]
default-features = false
features = ["2d"]
version = "=0.20.0"

[package]
edition = "2024"
name = "recipe-mesh"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use bevy::{color::palettes::basic::PURPLE, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform::default().with_scale(Vec3::splat(128.0)),
    ));
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/bevy/mesh/Cargo.toml
```

Run with `cargo run --locked --manifest-path site/downloads/bevy/mesh/Cargo.toml`. This requires a display and graphics environment. Compilation was verified; graphical runtime behavior was not checked.

Release evidence: [example, lines 3–24](https://docs.rs/crate/bevy/0.20.0/source/examples/2d/mesh2d.rs) · [documentation, lines 10–14](https://docs.rs/crate/bevy/0.20.0/source/docs/cargo_features.md) · [manifest, lines 138–139](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml.orig).

### Capture a custom CPU profiling span

Capture a scoped span on each graphical application update using trace\_chrome. Run in release mode with info-level spans enabled, then close the application normally and inspect the generated trace.

[Cargo.toml](examples/bevy/profile/Cargo.toml) · [Cargo.lock](examples/bevy/profile/Cargo.lock) · [src/main.rs](examples/bevy/profile/src/main.rs)

```toml
[dependencies.bevy]
default-features = true
features = ["trace_chrome"]
version = "=0.20.0"

[package]
edition = "2024"
name = "recipe-profile"
publish = false
version = "0.0.0"

[workspace]
```

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, profile_work)
        .run();
}

fn profile_work() {
    let _span = info_span!("example_work", name = "example_work").entered();
    let total: u64 = (0..1_000_u64)
        .map(std::hint::black_box)
        .sum();
    std::hint::black_box(total);
}
```

```sh
cargo check --locked --all-targets --manifest-path site/downloads/bevy/profile/Cargo.toml
```

Run with `cargo run --locked --release --manifest-path site/downloads/bevy/profile/Cargo.toml`. This requires a display and graphics environment. Compilation was verified; graphical runtime behavior was not checked.

Release evidence: [readme, lines 74–82](https://docs.rs/crate/bevy/0.20.0/source/README.md) · [api, lines 18–30](https://docs.rs/crate/bevy/0.20.0/source/src/lib.rs) · [documentation, lines 41–59](https://docs.rs/crate/bevy/0.20.0/source/docs/profiling.md) · [documentation, lines 131–135](https://docs.rs/crate/bevy/0.20.0/source/docs/profiling.md) · [manifest, lines 443–444](https://docs.rs/crate/bevy/0.20.0/source/Cargo.toml.orig).

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
