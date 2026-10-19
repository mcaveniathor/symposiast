use std::{path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut output = None;
    let mut base = "/symposiast/".to_owned();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "Usage: symposiast-site [OUTPUT] [--base /symposiast/]\n\nDefaults: OUTPUT=dist, --base=/symposiast/"
                );
                return ExitCode::SUCCESS;
            }
            "--base" => match args.next() {
                Some(value) => base = value,
                None => {
                    eprintln!("--base requires a URL path");
                    return ExitCode::FAILURE;
                }
            },
            _ if !arg.starts_with('-') && output.is_none() => output = Some(PathBuf::from(arg)),
            _ => {
                eprintln!("unexpected argument: {arg}");
                return ExitCode::FAILURE;
            }
        }
    }
    let output = output.unwrap_or_else(|| PathBuf::from("dist"));
    match symposiast_site::build(&output, &base) {
        Ok(files) => {
            println!("Wrote {} files to {}", files.len(), output.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Site build failed: {error}");
            ExitCode::FAILURE
        }
    }
}
