use crate::account::Account;
use crate::account_activity::AccountActivity;
use crate::processor::Processor;
use crate::processors::csv::reader::CsvReader;
use crate::processors::csv::writer::CsvWriter;
use crate::processors::csv::CsvProcessorError;
use std::collections::HashMap;
use std::io::{Read, Write};
use tracing::{error, warn};

pub struct CsvProcessor;

impl CsvProcessor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for CsvProcessor {
    type Error = CsvProcessorError;

    fn process_account_activity<I>(&self, activities: I) -> Vec<Account>
    where
        I: Iterator<Item=Result<AccountActivity, Self::Error>>,
    {
        let mut accounts = HashMap::new();
        for transaction in activities {
            match transaction {
                Err(err) => error!("error parsing account activity {:?}", err),
                Ok(transaction) => {
                    let account = accounts
                        .entry(transaction.client_id())
                        .or_insert_with(|| Account::new(transaction.client_id()));
                    if let Err(err) = account.transaction(transaction) {
                        warn!(
                            transaction_id = %transaction.transaction_id(),
                            client_id = %transaction.client_id(),
                            "error processing account activity: {}",err
                        );
                    }
                }
            }
        }
        accounts.into_values().collect()
    }

    fn process<R, W>(&self, input: R, output: W) -> Result<(), Self::Error>
    where
        R: Read,
        W: Write,
    {
        let mut reader = CsvReader::try_new(input)?;
        let accounts = self.process_account_activity(reader.iter());

        let mut writer = CsvWriter::new(output);
        writer.serialize(accounts.iter())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::processor::Processor;
    use crate::processors::csv::processor::CsvProcessor;
    use std::io::Cursor;

    struct TestCase<'a> {
        input: Vec<&'a str>,
        expected: Vec<&'a str>,
    }

    fn test(test_case: TestCase) {
        let input = Cursor::new(test_case.input.join("\n"));
        let mut output = Vec::new();
        let processor = CsvProcessor::new();

        let result = processor.process(input, &mut output);
        assert!(result.is_ok(), "Expected input processing to succeed: {:?}", result);

        let output = String::from_utf8(output).expect("Failed to convert output into string");
        let mut output = output.lines().collect::<Vec<&str>>();
        output.sort();

        let mut expected = test_case.expected;
        expected.sort();

        assert_eq!(output, expected);
    }

    #[test]
    fn process_account_activity() {
        test(TestCase {
            input: vec![
                "type,       client, tx, amount",
                "deposit,    1,      1,  100.0",
                "withdrawal, 1,      2,  24.5",
                "deposit,    2,      3,  100.0",
                "dispute,    1,      2",
                "withdrawal, 1,      4,  24.5",
                "dispute,    2,      3",
                "resolve,    1,      2",
                "withdrawal, 2,      5,  1000.0",
                "chargeback, 2,      3",
            ],
            expected: vec![
                "client,available,held,total,locked",
                "1,51.0,0.0,51.0,false",
                "2,0.0,0.0,0.0,true",
            ],
        })
    }
}