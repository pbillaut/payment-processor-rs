use crate::processor::Processor;
use crate::processors::csv::CsvProcessor;
use std::path::Path;

pub mod csv;

pub fn detect_processor(_path: &Path) -> impl Processor {
    // TODO: Logic to pick appropriate processor, e.g. based on MIME type.
    CsvProcessor::new()
}
