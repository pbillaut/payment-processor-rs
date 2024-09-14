use crate::account::Account;
use crate::account_activity::AccountActivity;
use std::error::Error;
use std::io;

pub trait Processor {
    type Error: Error;

    fn process_account_activity<I>(&self, activities: I) -> Vec<Account>
    where
        I: Iterator<Item=Result<AccountActivity, Self::Error>>;

    fn process<R, W>(&self, input: R, output: W) -> Result<(), Self::Error>
    where
        R: io::Read,
        W: io::Write;
}
