use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::ValueEnum;
use either::Either;

use grim_dawn_my_important_items::{Config, ProcessError, ValidateError, process, validate};
use thiserror::Error;

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum Mode {
    Standard,
    Diffs,
    Validate,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file. Reads from stdin if set to '-'.
    #[arg(short, long)]
    input: PathBuf,

    /// Path to the config TOML file.
    #[arg(long, default_value = "config.toml")]
    config: PathBuf,

    /// Mode: one of Standard, Diffs, or Validate.
    /// Diffs only outputs changes lines.
    /// Validate only validates that items in config appear in the input.
    #[arg(short, long, default_value = "standard")]
    mode: Mode,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{1}: {0}")]
    FileIO(std::io::Error, PathBuf),
    #[error("{1}: {0}")]
    Toml(toml::de::Error, PathBuf),
    #[error("process: {0}")]
    Process(#[from] ProcessError),
    #[error("validate: {0}")]
    Validate(#[from] ValidateError),
}

fn run(args: &Args) -> Result<(), RuntimeError> {
    let config =
        std::fs::read(&args.config).map_err(|e| RuntimeError::FileIO(e, args.config.clone()))?;

    let config = toml::from_slice::<Config>(&config)
        .map_err(|e| RuntimeError::Toml(e, args.config.clone()))?;

    let mut input = if args.input.as_os_str() == "-" {
        Either::Right(std::io::stdin().lock().lines())
    } else {
        let file =
            File::open(&args.input).map_err(|e| RuntimeError::FileIO(e, args.input.clone()))?;
        Either::Left(BufReader::new(file).lines())
    };

    let mut output = std::io::stdout().lock();

    if args.mode == Mode::Validate {
        validate(&mut input, &mut output, &config)?;
    } else {
        process(&mut input, &mut output, &config, args.mode == Mode::Diffs)?;
    }

    output.flush().map_err(RuntimeError::IO)?;

    Ok(())
}
