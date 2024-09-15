use criterion::criterion_main;

mod util;
mod benchmarks;

use payment_processor::account_activity::AccountActivity;
use payment_processor::processors::csv::CsvProcessorResult;

pub type ParseResult = Vec<CsvProcessorResult<AccountActivity>>;

pub const SCENARIOS: [(&str, u64); 2] = [
    ("activities_1K.csv", 1_000),
    ("activities_10K.csv", 10_000),
];

criterion_main!(
    benchmarks::parsing::benches,
    benchmarks::processing::benches,
);
