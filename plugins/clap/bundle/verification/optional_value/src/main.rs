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
