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
