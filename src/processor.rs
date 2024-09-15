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
                Err(err) => {
                    error!(error = ?err, "error parsing account activity record")
                }
                Ok(activity) => {
                    let account = accounts
                        .entry(activity.client_id())
                        .or_insert_with(|| Account::new(activity.client_id()));
                    if let Err(err) = account.transaction(activity) {
                        warn!(
                            activity = %activity,
                            transaction.id = %activity.transaction_id(),
                            client.id = %activity.client_id(),
                            error = ?err,
                            "error processing account activity",
                        );
                    }
                }
            }
        }
        accounts.into_values().collect()
    }
}
