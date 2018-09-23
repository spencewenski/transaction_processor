use serde_json;
use std::io;
use parser::{deserialize_keyed_items, Keyed, default_false, default_true};
use std::collections::{HashMap};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "accounts", deserialize_with = "deserialize_keyed_items")]
    pub accounts: HashMap<String, AccountConfig>,
    #[serde(rename = "categories", deserialize_with = "deserialize_keyed_items")]
    pub categories: HashMap<String, Category>,
    #[serde(rename = "formats", deserialize_with = "deserialize_keyed_items")]
    pub formats: HashMap<String, FormatConfig>,
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
    #[serde(rename = "payees", deserialize_with = "deserialize_keyed_items")]
    pub payees: HashMap<String, Payee>,
    #[serde(rename = "ignorePending", default = "default_true")]
    pub ignore_pending: bool,
    #[serde(rename = "sort")]
    pub sort: Option<Sort>,
}

impl Keyed<String> for AccountConfig {
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
        // Todo: figure out how to deserialize this directly into a Regex and also respect the ignoreCase option
        #[serde(rename = "matchString")]
        regex_string: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct Payee {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "categoryIds")]
    pub category_ids: Option<Vec<String>>,
}

impl Keyed<String> for Payee {
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

impl Keyed<String> for Category {
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

// todo: should FormatConfig go in the 'formats' module?
#[derive(Debug, Deserialize)]
pub struct FormatConfig {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "includeHeader", default = "default_true")]
    pub include_header: bool,
    #[serde(rename = "dataFormat")]
    pub data_format: DataFormat,
    #[serde(rename = "fieldOrder")]
    pub field_order: Vec<String>,
    #[serde(rename = "dateTimeConfig")]
    pub date_time_config: DateTimeConfig,
    #[serde(rename = "payeeConfig")]
    pub payee_config: PayeeConfig,
    #[serde(rename = "amountConfig")]
    pub amount_config: AmountConfig,
    #[serde(rename = "statusConfig")]
    pub status_config: Option<StatusConfig>,
    #[serde(rename = "memoConfig")]
    pub memo_config: Option<MemoConfig>,
    #[serde(rename = "categoryConfig")]
    pub category_config: Option<CategoryConfig>
}

impl Keyed<String> for FormatConfig {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

#[derive(Debug, Deserialize)]
pub enum DataFormat {
    #[serde(rename = "csv")]
    Csv,
}

#[derive(Debug, Deserialize)]
pub struct DateTimeConfig {
    #[serde(rename = "dateField")]
    pub date_field: String,
    #[serde(rename = "dateFormat")]
    pub date_format: String,
    #[serde(rename = "timeField")]
    pub time_field: Option<String>,
    #[serde(rename = "timeFormat")]
    pub time_format: Option<String>,
    #[serde(rename = "dateTimeDeliminator")]
    pub deliminator: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PayeeConfig {
    #[serde(rename = "fieldName")]
    pub field_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AmountConfig {
    #[serde(rename = "format")]
    pub format: AmountFormat,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AmountFormat {
    #[serde(rename = "SingleAmountField")]
    SingleAmountField(SingleAmountFieldConfig), // todo: create a test file for this
    #[serde(rename = "SeparateDebitCreditFields")]
    SeparateDebitCreditFields(SeparateDebitCreditFieldsConfig),
    #[serde(rename = "TransactionTypeAndAmountFields")]
    TransactionTypeAndAmountFields(TransactionTypeAndAmountFieldsConfig),
}

#[derive(Debug, Deserialize)]
pub struct SingleAmountFieldConfig {
    #[serde(rename = "fieldName")]
    pub field_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SeparateDebitCreditFieldsConfig {
    #[serde(rename = "debitField")]
    pub debit_field: String,
    #[serde(rename = "creditField")]
    pub credit_field: String,
}

#[derive(Debug, Deserialize)]
pub struct TransactionTypeAndAmountFieldsConfig {
    #[serde(rename = "amountField")]
    pub amount_field: String,
    #[serde(rename = "transactionTypeField")]
    pub transaction_type_field: String,
    #[serde(rename = "creditString")]
    pub credit_string: String,
    #[serde(rename = "debitString")]
    pub debit_string: String,
    #[serde(rename = "includeDebitSign", default = "default_false")]
    pub include_debit_sign: bool,
}

#[derive(Debug, Deserialize)]
pub struct StatusConfig {
    #[serde(rename = "fieldName")]
    pub field_name: String,
    #[serde(rename = "pendingString")]
    pub pending_string: String,
    #[serde(rename = "clearedString")]
    pub cleared_string: String,
}

#[derive(Debug, Deserialize)]
pub struct MemoConfig {
    #[serde(rename = "fieldName")]
    pub field_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CategoryConfig {
    #[serde(rename = "fieldName")]
    pub field_name: String,
}
