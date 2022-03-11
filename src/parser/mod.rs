use anyhow::anyhow;
use csv;
use csv::Writer;
use serde;
use serde::de::Error;
use serde::{de, Deserialize, Deserializer};
use std;
use std::collections::HashMap;
use std::io;
use std::str::FromStr;

pub fn parse_csv<T>() -> anyhow::Result<Vec<T>>
where
    for<'de> T: serde::Deserialize<'de>,
{
    parse_csv_from_reader(Box::new(io::stdin()))
}

pub fn parse_csv_from_string<T>(s: &'static str) -> anyhow::Result<Vec<T>>
where
    for<'de> T: serde::Deserialize<'de>,
{
    parse_csv_from_reader(Box::new(s.as_bytes()))
}

pub fn parse_csv_from_reader<T>(r: Box<dyn io::Read>) -> anyhow::Result<Vec<T>>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let mut reader = csv::Reader::from_reader(r);
    let mut values = Vec::new();
    for result in reader.deserialize() {
        match result {
            Ok(value) => values.push(value),
            Err(e) => return Err(anyhow!("An error occurred while parsing input: {}", e)),
        }
    }
    Ok(values)
}

pub fn write_csv<T>(values: Vec<T>, has_headers: bool) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    write_csv_to_writer(values, has_headers, Box::new(io::stdout()))
}

pub fn write_csv_to_writer<T>(
    values: Vec<T>,
    has_headers: bool,
    writer: Box<dyn io::Write>,
) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    let mut writer = create_csv_writer(has_headers, writer);
    for value in values {
        let r = writer.serialize(value);
        if let Err(e) = r {
            return Err(anyhow!("An error occurred while writing output: {}", e));
        }
    }
    Ok(())
}

pub fn create_csv_writer(
    has_headers: bool,
    writer: Box<dyn io::Write>,
) -> Writer<Box<dyn io::Write>> {
    csv::WriterBuilder::new()
        .has_headers(has_headers)
        .from_writer(writer)
}

pub fn deserialize_from_str<'de, D, T>(deserializer: D) -> anyhow::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
{
    let s = <String>::deserialize(deserializer)?;
    match T::from_str(&s) {
        Ok(v) => Ok(v),
        Err(_) => Err(de::Error::custom(format_args!(
            "unable to deserialize {}",
            s
        ))),
    }
}

pub fn deserialize_keyed_items<'de, D, K, V>(
    deserializer: D,
) -> anyhow::Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Keyed<K> + Deserialize<'de> + std::fmt::Display,
    K: std::hash::Hash + std::cmp::Eq + ToOwned + std::fmt::Display,
{
    let mut m: HashMap<K, V> = HashMap::new();
    let v: Vec<V> = Vec::deserialize(deserializer)?;
    for item in v {
        let key = item.key();
        if m.contains_key(&key) {
            return Err(D::Error::custom(format!(
                "Duplicate key [{}] for item [{}]",
                key, item
            )));
        }
        m.insert(key, item);
    }
    Ok(m)
}

pub trait Keyed<T> {
    fn key(&self) -> T;
}

pub fn default_true() -> bool {
    true
}

pub fn default_false() -> bool {
    false
}
