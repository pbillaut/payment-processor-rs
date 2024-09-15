//! A custom [`Deserialize`](serde::de::Deserialize) implementation that allows CSV records of
//! varying lengths to be deserialized into an enum of structs, addressing the limitation that the
//! [`csv`](csv) crate does currently not support this.
//!
//! For more details, see [this issue](https://github.com/BurntSushi/rust-csv/issues/211).
//!
use crate::account_activity::AccountActivity;
use crate::dispute::DisputeCase;
use crate::transaction::Transaction;
use serde::de::{Error, MapAccess};
use serde::{de, Deserialize};

impl<'de> Deserialize<'de> for AccountActivity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(RecordVisitor)
    }
}

struct RecordVisitor;

impl<'de> de::Visitor<'de> for RecordVisitor {
    type Value = AccountActivity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an account activity")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let _key = map.next_key::<&'de str>()?;
        let kind = map.next_value::<&'de str>()?;

        let variant = de::value::MapAccessDeserializer::new(map);
        match kind {
            "deposit" => Transaction::deserialize(variant).map(AccountActivity::Deposit),
            "withdrawal" => Transaction::deserialize(variant).map(AccountActivity::Withdrawal),
            "dispute" => DisputeCase::deserialize(variant).map(AccountActivity::Dispute),
            "resolve" => DisputeCase::deserialize(variant).map(AccountActivity::Resolve),
            "chargeback" => DisputeCase::deserialize(variant).map(AccountActivity::Chargeback),
            kind => Err(A::Error::unknown_variant(
                kind, &["deposit", "withdrawal", "dispute", "resolve", "chargeback"],
            )),
        }
    }
}
