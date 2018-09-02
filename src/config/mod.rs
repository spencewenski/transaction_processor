use serde_json;
use std::io;
use regex::Regex;
use parser::{deserialize_from_str, deserialize_item_with_key, Key, default_false, default_true};
use std::collections::{HashSet, HashMap};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "accounts", deserialize_with = "deserialize_item_with_key")]
    pub accounts: HashMap<String, AccountConfig>,
    #[serde(rename = "categories", deserialize_with = "deserialize_item_with_key")]
    pub categories: HashMap<String, Category>,
    #[serde(rename = "formats", deserialize_with = "deserialize_item_with_key")]
    pub formats: HashMap<String, Format>,
    #[serde(rename = "ignorePending", default = "default_true")]
    pub ignore_pending: bool,
    #[serde(rename = "skipPrompts", default = "default_false")]
    pub skip_prompts: bool,
    #[serde(rename = "sort")]
    pub sort: Option<Sort>,
}

impl Config {
    pub fn from_reader(r: Box<io::Read>) -> Result<Config, String> {
        Self::verify(serde_json::from_reader(r).unwrap())
    }

    fn verify(c: Config) -> Result<Config, String> {
        Ok(c)
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountConfig {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "formatId")]
    pub format_id: String,
    #[serde(rename = "payeeNormalizers")]
    pub payee_normalizers: Vec<PayeeNormalizerConfig>,
    #[serde(rename = "payees", deserialize_with = "deserialize_item_with_key")]
    pub payees: HashMap<String, Payee>,
    #[serde(rename = "ignorePending", default = "default_true")]
    pub ignore_pending: bool,
    #[serde(rename = "sort")]
    pub sort: Option<Sort>,
}

impl Key<String> for AccountConfig {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

#[derive(Debug, Deserialize)]
pub struct PayeeNormalizerConfig {
    #[serde(rename = "matcher")]
    pub normalizer_type: MatcherType,
    #[serde(rename = "payeeId")]
    pub payee_id: String,
    #[serde(rename = "ignoreCase", default = "default_true")]
    pub ignore_case: bool,
    #[serde(rename = "skipPrompts", default = "default_false")]
    pub skip_prompts: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MatcherType {
    #[serde(rename = "Exact")]
    Exact {
        #[serde(rename = "matchString")]
        exact_match_string: String,
    },
    #[serde(rename = "Contains")]
    Contains {
        #[serde(rename = "matchString")]
        contains_string: String,
    },
    #[serde(rename = "Regex")]
    Regex {
        #[serde(rename = "matchString", deserialize_with = "deserialize_from_str")]
        regex_string: Regex,
    },
}

#[derive(Debug, Deserialize)]
pub struct Payee {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "categoryIds")]
    pub category_ids: Option<HashSet<String>>,
}

impl Key<String> for Payee {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

#[derive(Debug, Deserialize)]
pub struct Category {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
}

impl Key<String> for Category {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

#[derive(Debug, Deserialize)]
pub struct Format {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "includeHeader", default = "default_true")]
    pub include_header: bool,
}

impl Key<String> for Format {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

#[derive(Debug, Deserialize)]
pub struct Sort {
    #[serde(rename = "sortBy")]
    pub sort_by: SortBy,
    #[serde(rename = "sortOrder")]
    pub order: SortOrder,
}

#[derive(Debug, Deserialize)]
pub enum SortBy {
    #[serde(rename = "date")]
    Date,
}

#[derive(Debug, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "ascending")]
    Ascending,
    #[serde(rename = "descending")]
    Descending,
}
