use serde_json;
use std::io;
use parser::{deserialize_keyed_items, Keyed, default_false, default_true};
use std::collections::{HashMap};
use config::arguments::{Arguments};
use util;

mod arguments;

pub struct Config {
    args: Arguments,
    config_file: ConfigFile,
}

impl Config {
    pub fn new_and_parse_args() -> Result<Config, String> {
        let args = Arguments::parse_args();
        let r = util::reader_from_file_name(&args.config_file)?;
        let config_file = ConfigFile::from_reader(r)?;
        Ok(Config {
            args,
            config_file,
        })
    }

    fn account_id(&self) -> &str {
        &self.args.src_account
    }

    pub fn account(&self) -> Option<&AccountConfig> {
        self.config_file.accounts.get(self.account_id())
    }

    fn dst_format_id(&self) -> &str {
        &self.args.dst_format
    }

    pub fn dst_format(&self) -> Option<&FormatConfig> {
        self.config_file.formats.get(self.dst_format_id())
    }

    pub fn src_file(&self) -> Option<&String> {
        self.args.src_file.as_ref()
    }

    pub fn dst_file(&self) -> Option<&String> {
        self.args.dst_file.as_ref()
    }

    pub fn category(&self, category_id: &str) -> Option<&Category> {
        self.config_file.categories.get(category_id)
    }

    pub fn sort(&self) -> Option<Sort> {
        self.args.sort.clone()
            .or(self.account().and_then(|a| {
                a.sort.clone()
            }))
            .or(self.config_file.sort.clone())
    }

    /// Whether to include the header in CSV output
    pub fn include_header(&self) -> bool {
        self.args.include_header
            .or(self.dst_format().and_then(|f| {
                f.include_header
            }))
            .unwrap_or(false)
    }

    pub fn ignore_pending(&self) -> bool {
        self.args.ignore_pending
            .or(self.account().and_then(|a| {
                a.ignore_pending
            }))
            .or(self.config_file.ignore_pending)
            .unwrap_or(true)
    }

    pub fn skip_prompts(&self) -> bool {
        self.args.skip_prompts
            .or(self.account().and_then(|a| {
                a.skip_prompts
            }))
            .or(self.config_file.skip_prompts)
            .unwrap_or(false)
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(rename = "accounts", deserialize_with = "deserialize_keyed_items")]
    accounts: HashMap<String, AccountConfig>,
    #[serde(rename = "categories", deserialize_with = "deserialize_keyed_items")]
    pub categories: HashMap<String, Category>,
    #[serde(rename = "formats", deserialize_with = "deserialize_keyed_items")]
    formats: HashMap<String, FormatConfig>,
    #[serde(rename = "ignorePending")]
    ignore_pending: Option<bool>,
    #[serde(rename = "skipPrompts")]
    skip_prompts: Option<bool>,
    #[serde(rename = "sort")]
    sort: Option<Sort>,
}

impl ConfigFile {
    fn from_reader(r: Box<io::Read>) -> Result<ConfigFile, String> {
        match serde_json::from_reader(r) {
            Ok(c) => Self::verify(c),
            Err(e) => Err(format!("Unable to read config file: {}", e)),
        }
    }

    fn verify(c: ConfigFile) -> Result<ConfigFile, String> {
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
    #[serde(rename = "ignorePending")]
    ignore_pending: Option<bool>,
    #[serde(rename = "sort")]
    sort: Option<Sort>,
    #[serde(rename = "skipPrompts")]
    skip_prompts: Option<bool>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct Sort {
    #[serde(rename = "sortBy")]
    pub sort_by: SortBy,
    #[serde(rename = "sortOrder")]
    pub order: SortOrder,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum SortBy {
    #[serde(rename = "date")]
    Date,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
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
    #[serde(rename = "includeHeader")]
    include_header: Option<bool>,
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
