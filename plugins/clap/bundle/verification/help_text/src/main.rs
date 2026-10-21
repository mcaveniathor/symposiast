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
