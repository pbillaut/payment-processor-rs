use crate::account::Account;
use crate::account_activity::AccountActivity;
use std::collections::HashMap;
use std::error::Error;
use tracing::{error, warn};

// TODO: This fn is public to be able to benchmark it. This should be handled with a bench feature
//       instead.
pub fn process_activities<I, E>(activities: I) -> Vec<Account>
where
    E: Error,
    I: Iterator<Item=Result<AccountActivity, E>>,
{
    let mut accounts = HashMap::new();
    for account_activity in activities {
        match account_activity {
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

pub trait Processor {
    type Error: Error;

    fn iter_input(&mut self) -> impl Iterator<Item=Result<AccountActivity, Self::Error>>;

    fn write(&mut self, accounts: Vec<Account>) -> Result<(), Self::Error>;

    fn process(&mut self) -> Result<(), Self::Error> {
        let activity_records = self.iter_input();
        let accounts = process_activities(activity_records);
        self.write(accounts)
    }
}
