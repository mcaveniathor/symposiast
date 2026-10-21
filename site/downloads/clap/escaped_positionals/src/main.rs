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
