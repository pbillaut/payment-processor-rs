// #![deny(clippy::missing_errors_doc)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::missing_assert_message)]

use std::fmt::{Display, Formatter};
use std::hash::Hash;

pub mod account_activity;
pub mod transaction;
pub mod dispute;

/// A globally unique client ID.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct ClientID(pub u16);

impl Display for ClientID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
