use crate::ParseResult;
use criterion::{black_box, criterion_group, BatchSize, Criterion};
use payment_processor::account_activity::AccountActivity;
use payment_processor::processor::Processor;
use payment_processor::processors::csv::CsvProcessor;
use payment_processor::transaction::TransactionID;
use payment_processor::ClientID;

fn bench_process_transactions(c: &mut Criterion) {
    let client_id_1 = ClientID(1);
    let client_id_2 = ClientID(2);
    let transactions = vec![
        AccountActivity::deposit(TransactionID(1), client_id_1, 100.0),
        AccountActivity::dispute(TransactionID(1), client_id_1),
        AccountActivity::deposit(TransactionID(2), client_id_1, 100.0),
        AccountActivity::withdrawal(TransactionID(3), client_id_1, 50.0),
        AccountActivity::resolve(TransactionID(1), client_id_1),
        AccountActivity::dispute(TransactionID(2), client_id_1),
        AccountActivity::chargeback(TransactionID(2), client_id_1),
        AccountActivity::withdrawal(TransactionID(4), client_id_1, 100.0),
        AccountActivity::deposit(TransactionID(1), client_id_2, 100.0),
        AccountActivity::dispute(TransactionID(1), client_id_2),
        AccountActivity::deposit(TransactionID(2), client_id_2, 100.0),
        AccountActivity::withdrawal(TransactionID(3), client_id_2, 50.0),
        AccountActivity::resolve(TransactionID(1), client_id_2),
        AccountActivity::dispute(TransactionID(2), client_id_2),
        AccountActivity::chargeback(TransactionID(2), client_id_2),
        AccountActivity::withdrawal(TransactionID(4), client_id_2, 100.0),
    ];
    let processor = CsvProcessor::new();

    c.bench_function("CsvProcessor::process_account_activity [dispute process]", move |b| {
        b.iter_batched(
            || transactions.clone().into_iter().map(Ok).collect::<ParseResult>(),
            |transactions| processor.process_account_activity(black_box(transactions.into_iter())),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_process_transactions);
