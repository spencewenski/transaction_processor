use std::io;
use csv;
use serde;

pub fn parse_csv<T>() -> Vec<T> where for<'de> T: serde::Deserialize<'de> {
    parse_csv_from_reader(io::stdin())
}

pub fn parse_csv_from_string<T>(s: &str) -> Vec<T> where for<'de> T: serde::Deserialize<'de> {
    parse_csv_from_reader(s.as_bytes())
}

pub fn parse_csv_from_reader<T, R>(r: R) -> Vec<T> where R: io::Read, for<'de> T: serde::Deserialize<'de> {
    let mut reader = csv::Reader::from_reader(r);
    let mut transactions = Vec::new();
    for result in reader.deserialize() {
        let transaction: T = result.unwrap();
        transactions.push(transaction);
    }
    transactions
}

pub fn write_csv<T>(transactions: Vec<T>, has_headers: bool) where T: serde::Serialize {
    write_csv_to_writer(transactions, has_headers, io::stdout())
}

pub fn write_csv_to_writer<T, W>(transactions: Vec<T>, has_headers: bool, writer: W) where W: io::Write, T: serde::Serialize {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(has_headers)
        .from_writer(writer);
    for transaction in transactions {
        writer.serialize(transaction);
    }
}