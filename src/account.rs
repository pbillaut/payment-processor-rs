use crate::account_activity::AccountActivity;
use crate::account_activity::AccountActivityError::{FailedDisputeCase, FailedTransaction, InvalidTransaction};
use crate::account_activity::AccountActivityResult;
use crate::transaction::{Transaction, TransactionID};
use crate::ClientID;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// A valid amount is neither negative (including -0.0), infinite, [subnormal] nor NaN.
///
/// [subnormal]: https://en.wikipedia.org/wiki/Denormal_number
fn is_valid_amount(amount: f32) -> bool {
    amount == 0.0 || amount.is_normal() && amount.is_sign_positive()
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    client_id: ClientID,

    available: f32,

    held: f32,

    total: f32,

    locked: bool,

    #[serde(skip)]
    dispute_cases: HashMap<TransactionID, f32>,

    #[serde(skip)]
    transaction_record: HashMap<TransactionID, Transaction>,
}

impl Account {
    pub fn new(client_id: ClientID) -> Self {
        Self {
            client_id,
            held: 0.0,
            total: 0.0,
            available: 0.0,
            locked: false,
            dispute_cases: HashMap::new(),
            transaction_record: HashMap::new(),
        }
    }

    /// Returns the total funds that are available.
    pub fn available(&self) -> f32 {
        self.available
    }

    /// Returns the total funds that disputed.
    pub fn held(&self) -> f32 {
        self.held
    }

    /// Returns the total funds that are [`available`](Self::available) or [`held`](Self::held).
    pub fn total(&self) -> f32 {
        self.total
    }

    /// Returns whether the account is locked.
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    fn lock(&mut self) {
        self.locked = true;
    }

    fn deposit(&mut self, amount: f32) -> AccountActivityResult<()> {
        if !is_valid_amount(amount) {
            return Err(InvalidTransaction("deposit amount must be a positive number".into()));
        }
        self.available += amount;
        self.total += amount;
        Ok(())
    }

    fn withdraw(&mut self, amount: f32) -> AccountActivityResult<()> {
        if !is_valid_amount(amount) {
            Err(InvalidTransaction("withdrawal amount must be a positive number".into()))
        } else if amount > self.available {
            Err(FailedTransaction("withdrawal failed because of insufficient funds".into()))
        } else {
            self.available -= amount;
            self.total -= amount;
            Ok(())
        }
    }

    fn hold(&mut self, amount: f32) -> AccountActivityResult<()> {
        if !is_valid_amount(amount) {
            Err(InvalidTransaction("hold amount must be a positive number".into()))
        } else {
            self.available -= amount;
            self.held += amount;
            Ok(())
        }
    }

    fn release(&mut self, amount: f32) -> AccountActivityResult<()> {
        if !is_valid_amount(amount) {
            Err(InvalidTransaction("release amount must be a positive number".into()))
        } else {
            self.held -= amount;
            self.available += amount;
            Ok(())
        }
    }

    fn charge_back(&mut self, amount: f32) -> AccountActivityResult<()> {
        if !is_valid_amount(amount) {
            Err(InvalidTransaction("chargeback amount must be a positive number".into()))
        } else {
            self.held -= amount;
            self.total -= amount;
            Ok(())
        }
    }

    fn initiate_dispute(&mut self, transaction_id: TransactionID) -> AccountActivityResult<()> {
        match self.dispute_cases.entry(transaction_id) {
            Entry::Occupied(_) => Err(FailedDisputeCase("transaction already disputed".into())),
            Entry::Vacant(entry) => {
                match self.transaction_record.get(&transaction_id) {
                    None => Ok(()),
                    Some(transaction) => {
                        entry.insert(transaction.amount());
                        self.hold(transaction.amount())
                    }
                }
            }
        }
    }

    fn resolve_dispute(&mut self, transaction_id: &TransactionID) -> AccountActivityResult<()> {
        match self.dispute_cases.remove(transaction_id) {
            None => Ok(()),
            Some(amount) => self.release(amount),
        }
    }

    fn issue_chargeback(&mut self, transaction_id: &TransactionID) -> AccountActivityResult<()> {
        match self.dispute_cases.remove(transaction_id) {
            None => Ok(()),
            Some(amount) => {
                self.charge_back(amount)?;
                self.lock();
                Ok(())
            }
        }
    }

    fn record_transaction(&mut self, transaction: Transaction) -> AccountActivityResult<()> {
        match self.transaction_record.entry(transaction.id()) {
            Entry::Occupied(_) => Err(FailedTransaction("transaction already processed".into())),
            Entry::Vacant(entry) => {
                entry.insert(transaction);
                Ok(())
            }
        }
    }

    pub fn transaction(&mut self, transaction: AccountActivity) -> AccountActivityResult<()> {
        if self.is_locked() {
            return Err(FailedTransaction("account locked".into()));
        }
        match transaction {
            AccountActivity::Deposit(transaction) => {
                self.deposit(transaction.amount())?;
                self.record_transaction(transaction)
            }
            AccountActivity::Withdrawal(transaction) => {
                self.withdraw(transaction.amount())?;
                self.record_transaction(transaction)
            }
            AccountActivity::Dispute(dispute_case) => {
                self.initiate_dispute(dispute_case.id())
            }
            AccountActivity::Resolve(dispute_case) => {
                self.resolve_dispute(&dispute_case.id())
            }
            AccountActivity::Chargeback(dispute_case) => {
                self.issue_chargeback(&dispute_case.id())
            }
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::Account;
    use crate::transaction::{Transaction, TransactionID};
    use crate::ClientID;
    use std::collections::HashMap;

    pub enum LockStatus {
        Locked,
        Unlocked,
    }

    impl Account {
        pub fn with_values(
            client_id: ClientID,
            available: f32,
            held: f32,
            total: f32,
            lock_status: LockStatus,
            transaction_record: HashMap<TransactionID, Transaction>,
            dispute_cases: HashMap<TransactionID, f32>,
        ) -> Self {
            Self {
                client_id,
                held,
                available,
                total,
                locked: match lock_status {
                    LockStatus::Locked => true,
                    LockStatus::Unlocked => false,
                },
                transaction_record,
                dispute_cases,
            }
        }
    }

    impl Default for Account {
        fn default() -> Self {
            Self::with_values(
                ClientID::default(),
                0.0,
                0.0,
                0.0,
                LockStatus::Unlocked,
                HashMap::new(),
                HashMap::new(),
            )
        }
    }
}

#[cfg(test)]
mod test_transactions {
    use super::Account;
    use crate::account_activity::AccountActivity;
    use crate::transaction::TransactionID;
    use crate::ClientID;

    #[test]
    fn dispute_affects_funds() {
        let deposit = AccountActivity::deposit(
            TransactionID::default(),
            ClientID::default(),
            100.0,
        );
        let dispute = AccountActivity::dispute(
            deposit.transaction_id(),
            deposit.client_id(),
        );

        let mut account = Account::default();
        account.transaction(deposit).expect("Test setup: deposit transaction failed");

        let result = account.transaction(dispute);
        assert!(result.is_ok(), "Expected dispute to succeed: {:?}: {:?}", dispute, result);
        assert_eq!(account.available(), 0.0);
        assert_eq!(account.held(), 100.0);
        assert_eq!(account.total(), 100.0);
    }

    #[test]
    fn dispute_of_non_existing_transaction_is_ignored() {
        let dispute = AccountActivity::dispute(
            TransactionID::default(),
            ClientID::default(),
        );

        let mut account_manager = Account::default();
        let result = account_manager.transaction(dispute);
        assert!(result.is_ok(),
                "Expected dispute of non-existing transaction to succeed: {:?}: {:?}",
                dispute, result);
    }

    #[test]
    fn disputing_multiple_different_transactions_is_possible() {
        let client_id = ClientID::default();
        let deposit_a = AccountActivity::deposit(
            TransactionID(0),
            client_id,
            50.0,
        );
        let deposit_b = AccountActivity::deposit(
            TransactionID(1),
            client_id,
            50.0,
        );

        let dispute_a = AccountActivity::dispute(
            deposit_a.transaction_id(),
            deposit_a.client_id(),
        );
        let dispute_b = AccountActivity::dispute(
            deposit_b.transaction_id(),
            deposit_b.client_id(),
        );

        let mut account = Account::default();
        account.transaction(deposit_a).expect("Test setup: deposit transaction failed");
        account.transaction(deposit_b).expect("Test setup: deposit transaction failed");

        let result = account.transaction(dispute_a);
        assert!(result.is_ok(),
                "Expected dispute to succeed: {:?}: {:?}", dispute_a, result);

        let result = account.transaction(dispute_b);
        assert!(result.is_ok(),
                "Expected dispute to succeed: {:?}: {:?}", dispute_b, result);

        assert_eq!(account.available(), 0.0);
        assert_eq!(account.held(), 100.0);
        assert_eq!(account.total(), 100.0);
    }

    #[test]
    fn disputing_same_transaction_twice_fails() {
        let deposit = AccountActivity::deposit(
            TransactionID::default(),
            ClientID::default(),
            50.0,
        );
        let dispute = AccountActivity::dispute(
            deposit.transaction_id(),
            deposit.client_id(),
        );

        let mut account = Account::default();
        account.transaction(deposit).expect("Test setup: deposit transaction failed");
        account.transaction(dispute).expect("Test setup: dispute failed");

        let result = account.transaction(dispute);
        assert!(result.is_err(),
                "Expected dispute on already disputed transaction to fail: {:?}", dispute);
    }

    #[test]
    fn resolve_affects_funds() {
        let deposit = AccountActivity::deposit(
            TransactionID::default(),
            ClientID::default(),
            50.0,
        );
        let dispute = AccountActivity::dispute(
            deposit.transaction_id(),
            deposit.client_id(),
        );
        let resolve = AccountActivity::resolve(
            deposit.transaction_id(),
            deposit.client_id(),
        );

        let mut account = Account::default();
        account.transaction(deposit).expect("Test setup: deposit transaction failed");
        account.transaction(dispute).expect("Test setup: dispute failed");

        let result = account.transaction(resolve);
        assert!(result.is_ok(), "Expected resolution to succeed: {:?}: {:?}", resolve, result);
        assert_eq!(account.available(), 50.0);
        assert_eq!(account.held(), 0.0);
    }

    #[test]
    fn resolve_for_non_existing_dispute_is_ignored() {
        let resolve = AccountActivity::resolve(TransactionID::default(), ClientID::default());

        let mut account = Account::default();
        let result = account.transaction(resolve);
        assert!(result.is_ok(),
                "Expected resolution of non-existent dispute case to succeed: {:?}: {:?}",
                resolve, result);
    }

    #[test]
    fn chargeback_affects_funds() {
        let deposit = AccountActivity::deposit(
            TransactionID::default(),
            ClientID::default(),
            50.0,
        );
        let dispute = AccountActivity::dispute(
            deposit.transaction_id(),
            deposit.client_id(),
        );
        let chargeback = AccountActivity::chargeback(
            deposit.transaction_id(),
            deposit.client_id(),
        );

        let mut account = Account::default();
        account.transaction(deposit).expect("Test setup: deposit transaction failed");
        account.transaction(dispute).expect("Test setup: dispute failed");

        let result = account.transaction(chargeback);
        assert!(result.is_ok(),
                "Expected chargeback to succeed: {:?}: {:?}", chargeback, result);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 0.0);
    }

    #[test]
    fn chargeback_for_non_existing_dispute_case_is_ignored() {
        let chargeback = AccountActivity::chargeback(TransactionID::default(), ClientID::default());

        let mut account = Account::default();
        let result = account.transaction(chargeback);
        assert!(result.is_ok(),
                "Expected chargeback for non-existent dispute case to succeed: {:?}: {:?}",
                chargeback, result);
    }

    #[test]
    fn chargeback_locks_account() {
        let deposit = AccountActivity::deposit(
            TransactionID::default(),
            ClientID::default(),
            50.0,
        );
        let dispute = AccountActivity::dispute(
            deposit.transaction_id(),
            deposit.client_id(),
        );
        let chargeback = AccountActivity::chargeback(
            deposit.transaction_id(),
            deposit.client_id(),
        );

        let mut account = Account::default();
        account.transaction(deposit).expect("Test setup: deposit transaction failed");
        account.transaction(dispute).expect("Test setup: dispute failed");
        account.transaction(chargeback).expect("Test setup: chargeback failed");

        assert!(account.is_locked(),
                "Expected account to be locked after successful chargeback");
    }
}

#[cfg(test)]
mod test_accounting {
    use super::*;

    #[test]
    fn deposit_affects_funds() {
        let amount = 100.0;

        let mut account = Account::default();

        let result = account.deposit(amount);
        assert!(result.is_ok(), "Expected deposit to succeed: {:?}", result);
        assert_eq!(account.available(), amount);
        assert_eq!(account.total(), amount);
    }

    #[test]
    fn deposit_with_invalid_value_fails() {
        let mut account = Account::default();

        let lower_than_min = 1.0e-40_f32;
        let invalid_values = [-1.0, lower_than_min, f32::NAN, f32::INFINITY];

        for invalid_value in invalid_values {
            let result = account.deposit(invalid_value);
            assert!(result.is_err(),
                    "Expected deposit with invalid value to fail: {:?}", invalid_value);
            assert_eq!(account.available(), 0.0);
            assert_eq!(account.total(), 0.0);
        }
    }

    #[test]
    fn withdrawal_affects_funds() {
        let amount = 100.0;

        let mut account = Account::default();
        account.deposit(amount).expect("Test setup: deposit failed");

        let result = account.withdraw(amount);
        assert!(result.is_ok(), "Expected withdrawal to succeed: {:?}", result);
        assert_eq!(account.available(), 0.0);
        assert_eq!(account.total(), 0.0);
    }

    #[test]
    fn withdraw_with_invalid_value_fails() {
        let mut account = Account::default();

        let lower_than_min = 1.0e-40_f32;
        let invalid_values = [-1.0, lower_than_min, f32::NAN, f32::INFINITY];

        for invalid_value in invalid_values {
            let result = account.withdraw(invalid_value);
            assert!(result.is_err(),
                    "Expected withdrawal with invalid value to fail: {:?}", invalid_value);
            assert_eq!(account.available(), 0.0);
            assert_eq!(account.total(), 0.0);
        }
    }

    #[test]
    fn withdraw_with_insufficient_funds_fails() {
        let available_funds = 100.0;

        let mut account = Account::default();
        account.deposit(available_funds).expect("Test setup: deposit failed");

        let result = account.withdraw(available_funds + 0.1);
        assert!(result.is_err(), "Expected withdrawal exceeding available funds to fail");
        assert_eq!(account.available(), available_funds);
        assert_eq!(account.total(), available_funds);
    }

    #[test]
    fn hold_affects_funds() {
        let amount = 100.0;

        let mut account = Account::default();
        account.deposit(amount).expect("Test setup: deposit failed");

        let result = account.hold(amount);
        assert!(result.is_ok(), "Expected hold to succeed: {:?}", result);
        assert_eq!(account.available(), 0.0);
        assert_eq!(account.held(), amount);
        assert_eq!(account.total(), amount);
    }

    #[test]
    fn hold_with_invalid_value_fails() {
        let mut account = Account::default();

        let lower_than_min = 1.0e-40_f32;
        let invalid_values = [-1.0, lower_than_min, f32::NAN, f32::INFINITY];

        for invalid_value in invalid_values {
            let result = account.hold(invalid_value);
            assert!(result.is_err(),
                    "Expected hold with invalid value to fail: {:?}", invalid_value);
            assert_eq!(account.available(), 0.0);
            assert_eq!(account.held(), 0.0);
            assert_eq!(account.total(), 0.0);
        }
    }

    #[test]
    fn release_affects_funds() {
        let amount = 100.0;

        let mut account = Account::default();
        account.deposit(amount).expect("Test setup: deposit failed");
        account.hold(amount).expect("Test setup: hold failed");

        let result = account.release(amount);
        assert!(result.is_ok(), "Expected release to succeed: {:?}", result);
        assert_eq!(account.available(), amount);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), amount);
    }

    #[test]
    fn release_with_invalid_value_fails() {
        let mut account = Account::default();

        let lower_than_min = 1.0e-40_f32;
        let invalid_values = [-1.0, lower_than_min, f32::NAN, f32::INFINITY];

        for invalid_value in invalid_values {
            let result = account.release(invalid_value);
            assert!(result.is_err(),
                    "Expected release with invalid value to fail: {:?}", invalid_value);
            assert_eq!(account.available(), 0.0);
            assert_eq!(account.held(), 0.0);
            assert_eq!(account.total(), 0.0);
        }
    }

    #[test]
    fn charge_back_affects_funds() {
        let amount = 100.0;

        let mut account = Account::default();
        account.deposit(amount).expect("Test setup: deposit failed");
        account.hold(amount).expect("Test setup: hold failed");

        let result = account.charge_back(amount);
        assert!(result.is_ok(), "Expected charge back to succeed: {:?}", result);
        assert_eq!(account.available(), 0.0);
        assert_eq!(account.held(), 0.0);
        assert_eq!(account.total(), 0.0);
    }

    #[test]
    fn charge_back_with_invalid_value_fails() {
        let mut account = Account::default();

        let lower_than_min = 1.0e-40_f32;
        let invalid_values = [-1.0, lower_than_min, f32::NAN, f32::INFINITY];

        for invalid_value in invalid_values {
            let result = account.charge_back(invalid_value);
            assert!(result.is_err(),
                    "Expected charge_back with invalid value to fail: {:?}", invalid_value);
            assert_eq!(account.available(), 0.0);
            assert_eq!(account.held(), 0.0);
            assert_eq!(account.total(), 0.0);
        }
    }
}
