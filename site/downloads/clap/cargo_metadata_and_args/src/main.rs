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
