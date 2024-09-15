use crate::ClientID;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

/// A globally unique transaction ID.
#[derive(serde::Deserialize, Debug, Eq, PartialEq, Clone, Copy, Hash, Default)]
pub struct TransactionID(pub u32);

impl Display for TransactionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A transaction is a financial activity or event where a value is exchanged between two parties.
/// In the context of banking or finance, it refers to any movement of funds involving accounts,
/// typically involving deposits, withdrawals, transfers, or payments.
#[derive(serde::Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Transaction {
    #[serde(rename = "tx")]
    id: TransactionID,

    #[serde(rename = "client")]
    client_id: ClientID,

    amount: Decimal,
}

impl Transaction {
    pub fn new(id: TransactionID, client_id: ClientID, amount: Decimal) -> Self {
        Self { id, client_id, amount }
    }

    pub fn id(&self) -> TransactionID {
        self.id
    }

    pub fn client_id(&self) -> ClientID {
        self.client_id
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }
}