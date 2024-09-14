use crate::processors::csv::CsvProcessorResult;
use serde::Serialize;
use std::io;

pub struct CsvWriter<W>
where
    W: io::Write,
{
    writer: csv::Writer<W>,
}

impl<W> CsvWriter<W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Self { writer: csv::Writer::from_writer(writer) }
    }

    pub fn serialize<S, I>(&mut self, records: I) -> CsvProcessorResult<()>
    where
        S: Serialize,
        I: Iterator<Item=S>,
    {
        for record in records {
            self.writer.serialize(record)?;
        }
        Ok(self.writer.flush()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::{test_utils::LockStatus, Account};
    use crate::ClientID;
    use std::collections::HashMap;

    #[test]
    fn serialize_account() {
        let account = Account::with_values(
            ClientID(101),
            10.0,
            20.0,
            30.0,
            LockStatus::Locked,
            HashMap::new(),
            HashMap::new(),
        );
        let expected = [
            "client,available,held,total,locked",
            "101,10.0,20.0,30.0,true",
        ].join("\n");

        let mut output = Vec::new();
        let result = {
            let mut writer = CsvWriter::new(&mut output);
            writer.serialize([account].iter())
        };
        let output = String::from_utf8(output).expect("Failed to convert output into string");

        assert!(result.is_ok(), "Expected serialization of account to succeed: {:?}", result);
        assert_eq!(output.trim(), expected.trim());
    }
}
