use crate::processors::csv::CsvProcessorError::InvalidFormat;
use crate::processors::csv::{CsvProcessorError, CsvProcessorResult};
use csv::{Reader, StringRecord, Trim};
use serde::de::DeserializeOwned;
use std::io;
use std::marker::PhantomData;

pub struct AccountActivityIter<'r, R: 'r, D> {
    reader: &'r mut Reader<R>,
    record: StringRecord,
    headers: StringRecord,
    phantom_data: PhantomData<D>,
}

impl<'r, R: io::Read, D: DeserializeOwned> AccountActivityIter<'r, R, D> {
    fn new(reader: &'r mut CsvReader<R>) -> AccountActivityIter<'r, R, D> {
        Self {
            reader: &mut reader.reader,
            record: StringRecord::new(),
            headers: reader.headers.clone(),
            phantom_data: PhantomData,
        }
    }
}

impl<'r, R: io::Read, D: DeserializeOwned> Iterator for AccountActivityIter<'r, R, D>
{
    type Item = CsvProcessorResult<D>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_record(&mut self.record) {
            Err(err) => Some(Err(err.into())),
            Ok(false) => None,
            Ok(true) => {
                let deserialized_record = if self.headers.len() > self.record.len() {
                    let mut headers = self.headers.clone();
                    headers.truncate(self.record.len());
                    self.record.deserialize(Some(&headers))
                } else {
                    self.record.deserialize(Some(&self.headers))
                };
                Some(deserialized_record.map_err(CsvProcessorError::Csv))
            }
        }
    }
}

pub struct CsvReader<R> {
    pub reader: Reader<R>,
    pub headers: StringRecord,
}

impl<R> CsvReader<R>
where
    R: io::Read,
{
    pub fn try_new(reader: R) -> CsvProcessorResult<Self> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .trim(Trim::All)
            .from_reader(reader);
        let headers = csv_reader
            .headers()
            .map_err(|_| InvalidFormat("invalid format: missing header line".into()))?
            .clone();
        Ok(Self { reader: csv_reader, headers })
    }

    pub fn iter<T>(&mut self) -> AccountActivityIter<R, T>
    where
        T: DeserializeOwned,
    {
        AccountActivityIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::CsvReader;
    use crate::account_activity::AccountActivity;
    use crate::processors::csv::CsvProcessorResult;
    use crate::transaction::TransactionID;
    use crate::ClientID;
    use rust_decimal_macros::dec;

    struct TestCase<'a> {
        input: Vec<&'a str>,
        expected: Vec<AccountActivity>,
    }

    fn test(test_case: TestCase) {
        let transactions = {
            let input = test_case.input.join("\n");
            let mut reader = CsvReader::try_new(input.as_bytes()).unwrap();
            reader
                .iter()
                .map(|r| r.unwrap())
                .collect::<Vec<AccountActivity>>()
        };
        assert_eq!(transactions, test_case.expected);
    }

    #[test]
    fn missing_headers_cause_error() {
        let input = "deposit, 1, 1, 100.0".to_string();
        let mut reader = CsvReader::try_new(input.as_bytes()).unwrap();
        let result = reader.iter().collect::<Vec<CsvProcessorResult<AccountActivity>>>();
        assert!(result.is_empty());
    }

    #[test]
    fn empty_input_is_valid() {
        test(TestCase {
            input: vec![],
            expected: vec![],
        })
    }

    #[test]
    fn zero_row_input_is_valid() {
        test(TestCase {
            input: vec!["type, client, tx, amount"],
            expected: vec![],
        })
    }

    #[test]
    fn transactions_and_disputes_are_serialized() {
        test(TestCase {
            input: vec![
                "type,       client, tx, amount",
                "deposit,    1,      1,  100.0",
                "deposit,    2,      2,  100.0",
                "dispute,    1,      1",
                "dispute,    2,      2",
                "resolve,    1,      1",
                "chargeback, 2,      2",
                "withdrawal, 1,      3,  100.0",
            ],
            expected: vec![
                AccountActivity::deposit(TransactionID(1), ClientID(1), dec!(100.0)),
                AccountActivity::deposit(TransactionID(2), ClientID(2), dec!(100.0)),
                AccountActivity::dispute(TransactionID(1), ClientID(1)),
                AccountActivity::dispute(TransactionID(2), ClientID(2)),
                AccountActivity::resolve(TransactionID(1), ClientID(1)),
                AccountActivity::chargeback(TransactionID(2), ClientID(2)),
                AccountActivity::withdrawal(TransactionID(3), ClientID(1), dec!(100.0)),
            ],
        })
    }

    #[test]
    fn transactions_are_serialized() {
        test(TestCase {
            input: vec![
                "type,       client, tx, amount",
                "deposit,    1,      1,  8.0",
                "withdrawal, 1,      2,  1.5",
                "withdrawal, 1,      3,  4.2",
            ],
            expected: vec![
                AccountActivity::deposit(TransactionID(1), ClientID(1), dec!(8.0)),
                AccountActivity::withdrawal(TransactionID(2), ClientID(1), dec!(1.5)),
                AccountActivity::withdrawal(TransactionID(3), ClientID(1), dec!(4.2)),
            ],
        })
    }

    #[test]
    fn disputes_are_serialized() {
        test(TestCase {
            input: vec![
                "type,       client, tx, amount",
                "dispute,    1,      1",
                "resolve,    1,      1",
                "dispute,    1,      2",
                "chargeback, 1,      2",
            ],
            expected: vec![
                AccountActivity::dispute(TransactionID(1), ClientID(1)),
                AccountActivity::resolve(TransactionID(1), ClientID(1)),
                AccountActivity::dispute(TransactionID(2), ClientID(1)),
                AccountActivity::chargeback(TransactionID(2), ClientID(1)),
            ],
        })
    }
}
