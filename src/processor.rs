use crate::account::Account;
use crate::account_activity::AccountActivity;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use tracing::{error, warn};

pub trait Processor {
    type Error: Error;

    fn process<R, W>(&self, input: R, output: W) -> Result<(), Self::Error>
    where
        R: io::Read,
        W: io::Write;

    fn process_account_activity<I>(&self, activities: I) -> Vec<Account>
    where
        I: Iterator<Item=Result<AccountActivity, Self::Error>>,
    {
        let mut accounts = HashMap::new();
        for transaction in activities {
            match transaction {
                Err(err) => error!("error parsing account activity {:?}", err),
                Ok(transaction) => {
                    let account = accounts
                        .entry(transaction.client_id())
                        .or_insert_with(|| Account::new(transaction.client_id()));
                    if let Err(err) = account.transaction(transaction) {
                        warn!(
                            transaction.id = %transaction.transaction_id(),
                            client.id = %transaction.client_id(),
                            "error processing account activity: {}",err
                        );
                    }
                }
            }
        }
        accounts.into_values().collect()
    }
}
