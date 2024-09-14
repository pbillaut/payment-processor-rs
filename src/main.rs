use clap::{Parser, ValueHint};
use payment_processor::processor::Processor;
use payment_processor::processors::detect_processor;
use std::error::Error;
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
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();

    let processor = detect_processor(&cli.path);
    let file = File::open(&cli.path)?;
    Ok(processor.process(file, io::stdout())?)
}
