use clap::{error::ErrorKind, Parser};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
}

fn main() {
    match Cli::try_parse() {
        Ok(cli) => println!("Selected port {}", cli.port),
        Err(error) => error.exit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_valid_port() {
        assert_eq!(Cli::try_parse_from(["server", "--port", "8080"]).unwrap().port, 8080);
    }

    #[test]
    fn rejects_invalid_and_zero_ports() {
        for value in ["zero", "0", "65536"] {
            let error = Cli::try_parse_from(["server", "--port", value]).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::ValueValidation);
        }
    }
}
