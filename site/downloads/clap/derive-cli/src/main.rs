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
