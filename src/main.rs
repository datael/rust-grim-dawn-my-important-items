use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use either::Either;

use grim_dawn_my_important_items::{Config, ProcessError, process};
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file. Reads from stdin if set to '-'.
    #[arg(short, long)]
    input: PathBuf,

    /// Path to the config TOML file.
    #[arg(long, default_value = "config.toml")]
    config: PathBuf,

    /// Diffs Only: only output changed lines.
    #[arg(short, long = "diffs")]
    diffs_only: bool,
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
}

fn run(args: &Args) -> Result<(), RuntimeError> {
    let config =
        std::fs::read(&args.config).map_err(|e| RuntimeError::FileIO(e, args.config.clone()))?;

    let config = toml::from_slice::<Config>(&config)
        .map_err(|e| RuntimeError::Toml(e, args.config.clone()))?;

    let mut input = if args.input.as_os_str() == OsStr::from_bytes(b"-") {
        Either::Right(std::io::stdin().lock().lines())
    } else {
        let file =
            File::open(&args.input).map_err(|e| RuntimeError::FileIO(e, args.input.clone()))?;
        Either::Left(BufReader::new(file).lines())
    };

    let mut output = std::io::stdout().lock();

    process(&mut input, &mut output, &config, args.diffs_only)?;

    output.flush().map_err(RuntimeError::IO)?;

    Ok(())
}
