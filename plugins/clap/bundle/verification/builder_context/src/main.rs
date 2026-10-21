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
