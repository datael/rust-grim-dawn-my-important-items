use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use either::Either;

use grim_dawn_my_important_items::{Config, process};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file. Reads from stdin if unspecified.
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file. Writes to stdout if unspecified.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Path to the config TOML file.
    #[arg(long, default_value = "config.toml")]
    config: PathBuf,

    /// Verbose: log a line to stderr for every change made.
    #[arg(short, long)]
    verbose: bool,

    /// Dry run: force output to be stdio even if an output is specified.
    #[arg(long)]
    dry: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(args: &Args) -> Result<(), String> {
    let config = std::fs::read(&args.config)
        .map_err(|e| format!("error reading config at {}: {e}", args.config.display()))?;

    let config = toml::from_slice::<Config>(&config)
        .map_err(|e| format!("error parsing config at {}: {e}", args.config.display()))?;

    let mut input = if let Some(path) = &args.input {
        let file = File::open(path)
            .map_err(|e| format!("error opening file at {}: {e}", path.display()))?;
        Either::Left(BufReader::new(file).lines())
    } else {
        Either::Right(std::io::stdin().lock().lines())
    };

    let mut output = if !args.dry
        && let Some(path) = &args.output
    {
        let file = File::create(path)
            .map_err(|e| format!("error creating file at {}: {e}", path.display()))?;
        Either::Left(BufWriter::new(file))
    } else {
        Either::Right(std::io::stdout().lock())
    };

    process(&mut input, &mut output, &config, args.verbose)?;

    output
        .flush()
        .map_err(|e| format!("error flushing output: {e}"))?;

    Ok(())
}
