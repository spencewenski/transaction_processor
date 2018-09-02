use std::io;
use csv;
use serde;
use serde::{Deserializer, Deserialize};
use std::str::FromStr;
use serde::de;
use std::collections::HashMap;
use std;

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

pub fn deserialize_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where D: Deserializer<'de>, T: FromStr
{
    let s = <String>::deserialize(deserializer)?;
    match T::from_str(&s) {
        Ok(v) => Ok(v),
        Err(_) => Err(de::Error::custom(format_args!("unable to deserialize {}", s))),
    }
}

pub fn deserialize_item_with_key<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
    where D: Deserializer<'de>, V: Key<K> + Deserialize<'de>, K: std::hash::Hash + std::cmp::Eq + ToOwned
{
    let mut m : HashMap<K, V> = HashMap::new();
    let v : Vec<V> = Vec::deserialize(deserializer)?;
    for item in v {
        m.insert(item.key(), item);
    }
    Ok(m)
}

pub trait Key<T> {
    fn key(&self) -> T;
}

pub fn default_true() -> bool {
    true
}

pub fn default_false() -> bool {
    false
}