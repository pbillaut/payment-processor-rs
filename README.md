# Payment Processor

## Performance

Since no target has been specified in terms of performance, this processor has been optimized for robustness and
convenience. Performance has been still considered where it makes sense, as to not leave any obvious optimizations on
the table.

### Parsing

Unsurprisingly, the slowest part of data processing is reading the CSV file for parsing, since we're waiting for I/O
most of the time. Unfortunately, the popular [csv](https://docs.rs/csv) crate doesn't offer reading files asynchronously
and optimization potential here is limited. Alternative crates providing async reading exist, but need to be evaluated.

### Calculations

See [Bench Report](docs/bench-reports/decimals).