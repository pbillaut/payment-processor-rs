use crate::util::{open_file, read_file};
use crate::{ParseResult, SCENARIOS};
use criterion::{black_box, criterion_group, BenchmarkId, Criterion};
use payment_processor::processors::csv::reader::CsvReader;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("CsvReader::iter");
    for (filename, num_elements) in SCENARIOS {
        let buffer = read_file!(filename);
        group.throughput(criterion::Throughput::Elements(num_elements));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_elements), &buffer,
            |b, buffer| b.iter(|| {
                let mut reader = CsvReader::try_new(black_box(buffer.as_slice()))
                    .expect("Benchmark: unable to create csv reader");
                reader.iter().collect::<ParseResult>()
            }),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_parsing);
