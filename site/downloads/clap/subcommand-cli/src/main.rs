use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about = "Manage a small task list")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Add { name: String },
    List {
        #[arg(long)]
        all: bool,
    },
}

fn main() {
    match Cli::parse().command {
        Command::Add { name } => println!("Add {name}"),
        Command::List { all } => println!("List tasks (include completed: {all})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_each_variant() {
        let cli = Cli::try_parse_from(["tasks", "add", "write tests"]).unwrap();
        assert!(matches!(cli.command, Command::Add { name } if name == "write tests"));
        let cli = Cli::try_parse_from(["tasks", "list", "--all"]).unwrap();
        assert!(matches!(cli.command, Command::List { all: true }));
    }

    #[test]
    fn requires_a_subcommand() {
        assert!(Cli::try_parse_from(["tasks"]).is_err());
    }
}
