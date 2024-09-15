# Payment Processor

A simple toy payment processor.

## A Note on Parsing

In parsing a CSV file containing different types of records, such as transactions and dispute events, using separate
structs for each type, united into an enum (see [`AccountActivity`](src/account_activity.rs)), offers significant
advantages over a single struct with optional fields.

Although the [csv][crate:csv] crate doesn't natively support parsing into this type of data structure, the
effort to adapt the deserializer is worthwhile. It enhances safety, reduces runtime errors, and improves code clarity
in the long term.

### Type-Level Invariants

Separate structs ensure that each record type only contains its relevant fields. A transaction struct will always have
fields transaction id, client id and amount, whereas a dispute event will only include transaction id and client id.
This enforces clear, type-safe invariants, preventing errors caused by missing or irrelevant fields.

### Compile-Time Safety

The Rust compiler can enforce the correctness of the data structures, eliminating the need for checking `Options` and
reducing runtime errors. Each record type is guaranteed to have only the fields it needs, providing strong compile-time
guarantees.

## Performance

As no specific performance target has been set, the processor is primarily optimized for robustness and convenience.
However, performance has still been considered where appropriate to avoid missing obvious optimizations.

### Parsing

As expected, the slowest part of data processing is CSV file parsing, primarily due to I/O waiting times. The commonly
used [`csv`][crate:csv] crate does not support asynchronous file reading, limiting optimization potential in
this area. While alternative crates with async support exist, they require further evaluation before adoption.

### Calculations

Although [benchmarks](docs/bench-reports/decimals) indicate that the use of the [`Decimal`][type:decimal] type of the
[`rust_decimal`][crate:rust_decimal] crate results in approximately a 20% performance decrease, its
benefits make it a sensible choice for financial calculations.

The crate ensures there are no rounding errors, which is crucial when dealing with financial data. Additionally, common
issues associated with floating point types, such as NaN, infinite, or subnormal values, are avoided since decimals
inherently cannot represent these states, eliminating the need for additional checks.

As above, since no specific performance target has been set, there is no strong justification for exploring alternatives
that rely on native floating-point types. The advantages provided by [`rust_decimal`][crate:rust_decimal], particularly
in terms of precision and error avoidance, outweigh the potential performance gains from using floats.

[type:decimal]: https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html

[crate:csv]: https://docs.rs/csv/latest

[crate:rust_decimal]: https://docs.rs/rust_decimal/latest

