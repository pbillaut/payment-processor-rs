use anyhow::Context;
use clap::{Parser, ValueHint};
use payment_processor::processor::Processor;
use payment_processor::processors::csv::CsvProcessor;
use std::io::Write;
use std::{fs::File, io, path::PathBuf};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a file that holds account activity records.
    ///
    /// Supported file formats: CSV
    #[arg(value_hint = ValueHint::FilePath)]
    path: PathBuf,

    /// Whether to suppress printing the results to stdout.
    #[clap(long, action)]
    silent: bool,
}

fn output(silent: bool) -> Box<dyn Write> {
    if silent { Box::new(io::sink()) } else { Box::new(io::stdout()) }
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let file = File::open(&cli.path).context("unable to open file input file")?;

    let mut processor = CsvProcessor::try_new(file, output(cli.silent))?;
    processor.process().context("processing input file failed")
}
