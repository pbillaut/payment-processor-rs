use crate::account::Account;
use crate::account_activity::AccountActivity;
use std::collections::HashMap;
use std::error::Error;
use tracing::{error, warn};

// TODO: This fn is public to be able to benchmark it. This should probably be handled with a bench 
//       feature instead.
pub fn process_activities<I, E>(activities: I) -> Vec<Account>
where
    E: Error,
    I: Iterator<Item=Result<AccountActivity, E>>,
{
    let mut accounts = HashMap::new();
    for account_activity in activities {
        match account_activity {
            Err(err) => {
                warn!(error = ?err, "error parsing account activity record")
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

#[cfg(test)]
mod tests {
    use crate::account::test_utils::LockStatus;
    use crate::account::Account;
    use crate::account_activity::AccountActivity;
    use crate::processor::process_activities;
    use crate::processor::tests::DummyError::ParseError;
    use crate::transaction::TransactionID;
    use crate::ClientID;
    use thiserror::Error;

    #[derive(Error, Debug, Clone)]
    pub enum DummyError {
        #[error("error parsing account activity record")]
        ParseError,
    }

    struct TestCase {
        activities: Vec<Result<AccountActivity, DummyError>>,
        expected: Vec<Account>,
    }

    fn test(test_case: TestCase) {
        let output = process_activities(test_case.activities.into_iter());
        for (account, expected) in output.into_iter().zip(test_case.expected) {
            assert_eq!(account.client_id(), expected.client_id());
            assert_eq!(account.available(), expected.available());
            assert_eq!(account.held(), expected.held());
            assert_eq!(account.total(), expected.total());
            assert_eq!(account.is_locked(), expected.is_locked());
        }
    }

    #[test]
    fn erroneous_records_are_skipped() {
        test(
            TestCase {
                activities: vec![
                    Ok(AccountActivity::deposit(
                        TransactionID::default(),
                        ClientID::default(),
                        10.0
                    )),
                    // The next record couldn't be parsed
                    Err(ParseError),
                    Ok(AccountActivity::withdrawal(
                        TransactionID::default(),
                        ClientID::default(),
                        5.0
                    ))
                ],
                expected: vec![
                    Account::with_values(
                        ClientID::default(),
                        5.0,
                        0.0,
                        5.0,
                        LockStatus::Unlocked
                    )
                ],
            }
        )
    }

    #[test]
    fn invalid_activities_are_skipped() {
        test(
            TestCase {
                activities: vec![
                    Ok(AccountActivity::deposit(
                        TransactionID::default(),
                        ClientID::default(),
                        10.0
                    )),
                    // The next activity should cause an insufficient funds error
                    Ok(AccountActivity::withdrawal(
                        TransactionID::default(),
                        ClientID::default(),
                        15.0
                    )),
                    Ok(AccountActivity::withdrawal(
                        TransactionID::default(),
                        ClientID::default(),
                        10.0
                    )),
                ],
                expected: vec![
                    Account::with_values(
                        ClientID::default(),
                        0.0,
                        0.0,
                        0.0,
                        LockStatus::Unlocked
                    )
                ],
            }
        )
    }
}
