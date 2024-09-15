use crate::account::Account;
use crate::account_activity::AccountActivity;
use crate::processor::Processor;
use crate::processors::csv::reader::CsvReader;
use crate::processors::csv::writer::CsvWriter;
use crate::processors::csv::CsvProcessorError;
use std::io::{Read, Write};

pub struct CsvProcessor<R, W>
where
    R: Read,
    W: Write,
{
    reader: CsvReader<R>,
    writer: CsvWriter<W>,
}

impl<R, W> CsvProcessor<R, W>
where
    R: Read,
    W: Write,
{
    pub fn try_new(input: R, output: W) -> Result<Self, anyhow::Error> {
        let reader = CsvReader::try_new(input)?;
        let writer = CsvWriter::new(output);
        Ok(Self {
            reader,
            writer,
        })
    }
}

impl<R, W> Processor for CsvProcessor<R, W>
where
    R: Read,
    W: Write,
{
    type Error = CsvProcessorError;

    fn iter_input(&mut self) -> impl Iterator<Item=Result<AccountActivity, Self::Error>> {
        self.reader.iter()
    }

    fn write(&mut self, accounts: Vec<Account>) -> Result<(), Self::Error> {
        self.writer.serialize(accounts.iter())
    }
}
