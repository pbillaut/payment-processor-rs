use criterion::criterion_main;

mod benchmarks;

pub type ParseResult = Vec<CsvProcessorResult<AccountActivity>>;

#[allow(unused_macros)]
macro_rules! open_file {
    ($str_arg:expr) => {{
        let mut path = PathBuf::from(file!());
        path.pop();
        path.pop();
        path.push("data/");
        path.push($str_arg);
        File::open(path).expect("Benchmark setup: unable to open file")
    }};
}
#[allow(unused_imports)]
pub(crate) use open_file;
use payment_processor::account_activity::AccountActivity;
use payment_processor::processors::csv::CsvProcessorResult;

#[allow(unused_macros)]
macro_rules! read_file {
    ($str_arg:expr) => {{
        let mut file = open_file!($str_arg);
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Benchmark setup: unable to read file");
        buffer
    }};
}
#[allow(unused_imports)]
pub(crate) use read_file;

pub const SCENARIOS: [(&str, u64); 2] = [
    ("activities_1K.csv", 1_000),
    ("activities_10K.csv", 10_000),
];

criterion_main!(
    benchmarks::parsing::benches,
    benchmarks::processing::benches,
);
