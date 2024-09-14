use crate::transaction::TransactionID;
use crate::ClientID;

/// A dispute case is a formal challenge or objection raised by a customer or client regarding the
/// accuracy or legitimacy of a particular transaction.
///
/// A dispute cases goes through the following process:
///
/// 1. __Initiation__: The account holder contacts their bank or financial institution to raise a
///    concern about the transaction.
/// 2. __Investigation__: Once the dispute is initiated, the bank or financial institution
///    investigates the transaction. They might review transaction logs, check for evidence of
///    fraud, or verify whether the account holder authorized the withdrawal.
/// 3. __Resolution__: Based on the investigation, the financial institution may either:
///    1. __Chargeback__: Reverse or refund the transaction if the dispute is found valid.
///    2. __Resolve__: Uphold the transaction if the bank determines it was correctly processed, and 
///       the dispute is not valid.
#[derive(serde::Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub struct DisputeCase {
    #[serde(rename = "tx")]
    transaction_id: TransactionID,

    #[serde(rename = "client")]
    client_id: ClientID,
}

impl DisputeCase {
    pub fn new(id: TransactionID, client_id: ClientID) -> Self {
        Self { transaction_id: id, client_id }
    }

    pub fn id(&self) -> TransactionID {
        self.transaction_id
    }

    pub fn client_id(&self) -> ClientID {
        self.client_id
    }
}
