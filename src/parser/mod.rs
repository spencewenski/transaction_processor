use std::io;
use csv;
use serde;

pub fn parse_csv<T>() -> Vec<T> where for<'de> T: serde::Deserialize<'de> {
    parse_csv_from_reader(Box::new(io::stdin()))
}

pub fn parse_csv_from_string<T>(s: &'static str) -> Vec<T> where for<'de> T: serde::Deserialize<'de> {
    parse_csv_from_reader(Box::new(s.as_bytes()))
}

pub fn parse_csv_from_reader<T>(r: Box<io::Read>) -> Vec<T> where for<'de> T: serde::Deserialize<'de> {
    let mut reader = csv::Reader::from_reader(r);
    let mut values = Vec::new();
    for result in reader.deserialize() {
        let value: T = result.unwrap();
        values.push(value);
    }
    values
}

pub fn write_csv<T>(values: Vec<T>, has_headers: bool) where T: serde::Serialize {
    write_csv_to_writer(values, has_headers, Box::new(io::stdout()))
}

pub fn write_csv_to_writer<T>(values: Vec<T>, has_headers: bool, writer: Box<io::Write>) where T: serde::Serialize {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(has_headers)
        .from_writer(writer);
    for value in values {
        writer.serialize(value).unwrap();
    }
}