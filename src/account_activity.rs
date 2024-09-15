use crate::dispute::DisputeCase;
use crate::transaction::{Transaction, TransactionID};
use crate::ClientID;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountActivityError {
    /// Indicates that the payload of a transaction is invalid.
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Indicates that a transaction could not be executed.
    ///
    /// This covers cases such as an attempt of a withdrawal with insufficient funds or an internal
    /// processing error.
    #[error("failed transaction: {0}")]
    FailedTransaction(String),

    /// Indicates that a dispute case could not be executed.
    ///
    /// This covers cases such as a dispute being initiated on an already disputed transaction.
    #[error("failed dispute case: {0}")]
    FailedDisputeCase(String),
}

pub type AccountActivityResult<T> = Result<T, AccountActivityError>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AccountActivity {
    /// A [`Transaction`] where funds are added to an account, increasing the available and total
    /// balance of the account.
    ///
    /// Deposits can be made in various ways, such as transferring money from another account,
    /// depositing cash, or receiving payments.
    Deposit(Transaction),

    /// A [`Transaction`] where funds are removed from an account, reducing the available and total
    /// account balance.
    ///
    /// Withdrawals can be made in various ways, such as cash withdrawals, bank transfers to other
    /// accounts, or using checks or debit cards for purchases.
    Withdrawal(Transaction),

    /// A dispute is the initiation of a [`DisputeCase`].
    ///
    /// It is a formal objection raised by a customer regarding a particular [`Transaction`].
    ///
    /// This typically occurs when the customer believes the transaction is unauthorized,
    /// fraudulent, or processed incorrectly (e.g., an incorrect amount or duplicate charge).
    ///
    /// The dispute triggers an investigation by the bank or financial institution to determine the
    /// validity of the claim.
    Dispute(DisputeCase),

    /// A resolve or resolution is the continuation of a [`DisputeCase`].
    ///
    /// It is the process of addressing and concluding a [`Dispute`]. During the resolution process,
    /// the financial institution investigates the disputed transaction, reviews evidence, and makes
    /// a decision on whether the claim is valid.
    ///
    /// A resolution can result in the dispute being upheld (leading to a refund or chargeback) or
    /// denied.
    ///
    /// [`Dispute`]: AccountActivity::Dispute
    Resolve(DisputeCase),

    /// A chargeback is the continuation of a [`DisputeCase`].
    ///
    /// It is a refund issued to a customer after a [`Dispute`] is resolved in their favor,
    /// reversing the funds transferred in the disputed transaction back to the claimant's account.
    ///
    /// Chargebacks are typically initiated by the bank or payment processor and are used to protect
    /// customers from fraudulent or erroneous charges.
    ///
    /// [`Dispute`]: AccountActivity::Dispute
    Chargeback(DisputeCase),
}

impl AccountActivity {
    pub fn deposit(transaction_id: TransactionID, client_id: ClientID, amount: Decimal) -> Self {
        Self::Deposit(Transaction::new(transaction_id, client_id, amount))
    }

    pub fn withdrawal(transaction_id: TransactionID, client_id: ClientID, amount: Decimal) -> Self {
        Self::Withdrawal(Transaction::new(transaction_id, client_id, amount))
    }

    pub fn dispute(transaction_id: TransactionID, client_id: ClientID) -> Self {
        Self::Dispute(DisputeCase::new(transaction_id, client_id))
    }

    pub fn resolve(transaction_id: TransactionID, client_id: ClientID) -> Self {
        Self::Resolve(DisputeCase::new(transaction_id, client_id))
    }

    pub fn chargeback(transaction_id: TransactionID, client_id: ClientID) -> Self {
        Self::Chargeback(DisputeCase::new(transaction_id, client_id))
    }

    pub fn transaction_id(&self) -> TransactionID {
        match self {
            AccountActivity::Deposit(transaction) => transaction.id(),
            AccountActivity::Withdrawal(transaction) => transaction.id(),
            AccountActivity::Dispute(transaction) => transaction.id(),
            AccountActivity::Resolve(transaction) => transaction.id(),
            AccountActivity::Chargeback(transaction) => transaction.id(),
        }
    }

    pub fn client_id(&self) -> ClientID {
        match self {
            AccountActivity::Deposit(transaction) => transaction.client_id(),
            AccountActivity::Withdrawal(transaction) => transaction.client_id(),
            AccountActivity::Dispute(transaction) => transaction.client_id(),
            AccountActivity::Resolve(transaction) => transaction.client_id(),
            AccountActivity::Chargeback(transaction) => transaction.client_id(),
        }
    }
}

impl Display for AccountActivity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            AccountActivity::Deposit(_) => "deposit",
            AccountActivity::Withdrawal(_) => "withdrawal",
            AccountActivity::Dispute(_) => "dispute",
            AccountActivity::Resolve(_) => "resolve",
            AccountActivity::Chargeback(_) => "chargeback",
        };
        write!(f, "{}", kind)
    }
}
