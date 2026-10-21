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
