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
