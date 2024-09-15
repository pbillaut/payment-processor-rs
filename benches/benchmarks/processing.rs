use crate::ParseResult;
use criterion::{black_box, criterion_group, BatchSize, Criterion};
use payment_processor::account_activity::AccountActivity;
use payment_processor::processor::process_activities;
use payment_processor::transaction::TransactionID;
use payment_processor::ClientID;
use rust_decimal_macros::dec;

fn bench_process_transactions(c: &mut Criterion) {
    let client_id_1 = ClientID(1);
    let client_id_2 = ClientID(2);
    let transactions = vec![
        AccountActivity::deposit(TransactionID(1), client_id_1, dec!(100.0)),
        AccountActivity::dispute(TransactionID(1), client_id_1),
        AccountActivity::deposit(TransactionID(2), client_id_1, dec!(100.0)),
        AccountActivity::withdrawal(TransactionID(3), client_id_1, dec!(50.0)),
        AccountActivity::resolve(TransactionID(1), client_id_1),
        AccountActivity::dispute(TransactionID(2), client_id_1),
        AccountActivity::chargeback(TransactionID(2), client_id_1),
        AccountActivity::withdrawal(TransactionID(4), client_id_1, dec!(100.0)),
        AccountActivity::deposit(TransactionID(1), client_id_2, dec!(100.0)),
        AccountActivity::dispute(TransactionID(1), client_id_2),
        AccountActivity::deposit(TransactionID(2), client_id_2, dec!(100.0)),
        AccountActivity::withdrawal(TransactionID(3), client_id_2, dec!(50.0)),
        AccountActivity::resolve(TransactionID(1), client_id_2),
        AccountActivity::dispute(TransactionID(2), client_id_2),
        AccountActivity::chargeback(TransactionID(2), client_id_2),
        AccountActivity::withdrawal(TransactionID(4), client_id_2, dec!(100.0)),
    ];

    c.bench_function("process_activities [dispute process]", move |b| {
        b.iter_batched(
            || transactions.clone().into_iter().map(Ok).collect::<ParseResult>(),
            |transactions| process_activities(black_box(transactions.into_iter())),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_process_transactions);
