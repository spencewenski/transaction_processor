use serde_json;
use std::io;
use parser::{deserialize_keyed_items, Keyed, default_false, default_true};
use std::collections::{HashMap};
use config::arguments::{Arguments};
use util;
use regex::RegexBuilder;
use itertools::Itertools;
use std::fmt;
use std::fmt::Display;

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
        validate_args(&args, &config_file)?;
        Ok(Config {
            args,
            config_file,
        })
    }

    fn account_id(&self) -> &str {
        &self.args.src_account
    }

    pub fn account(&self) -> &AccountConfig {
        // We validated the input, so this should never return Option::None
        self.config_file.accounts.get(self.account_id())
            .expect(&format!("Account [{}] does not exist.", self.account_id()))
    }

    pub fn src_format(&self) -> &FormatConfig {
        // We validated the input, so this should never return Option::None
        self.config_file.formats.get(&self.account().format_id)
            .expect(&format!("Source format [{}] does not exist.", self.account().format_id))
    }

    pub fn dst_format_id(&self) -> &str {
        &self.args.dst_format
    }

    pub fn dst_format(&self) -> &FormatConfig {
        // We validated the input, so this should never return Option::None
        self.config_file.formats.get(self.dst_format_id())
            .expect(&format!("Destination format [{}] does not exist.", self.dst_format_id()))
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
            .or(self.account().sort.clone())
            .or(self.config_file.sort.clone())
    }

    /// Whether to include the header in CSV output
    pub fn include_header(&self) -> bool {
        self.args.include_header
            .or(self.dst_format().include_header)
            .unwrap_or(false)
    }

    pub fn ignore_pending(&self) -> bool {
        self.args.ignore_pending
            .or(self.account().ignore_pending)
            .or(self.config_file.ignore_pending)
            .unwrap_or(false)
    }

    pub fn skip_prompts(&self) -> bool {
        self.args.skip_prompts
            .or(self.account().skip_prompts)
            .or(self.config_file.skip_prompts)
            .unwrap_or(false)
    }
}

fn validate_args(args: &Arguments, c: &ConfigFile) -> Result<(), String> {
    if !c.accounts.contains_key(&args.src_account) {
        return Err(format!("Source account id [{}] specified on command line does not exist in config file. Available options are [{}].",
                           args.src_account, c.accounts.iter().map(|(_, a)| &a.id).sorted().iter().join(", ")));
    }
    if !c.formats.contains_key(&args.dst_format) {
        return Err(format!("Destination format id [{}] specified on command line does not exist in config file. Available options are [{}].",
                           args.dst_format, c.formats.iter().map(|(_, f)| &f.id).sorted().iter().join(", ")));
    }
    Ok(())
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
            Ok(c) => Self::validate(c),
            Err(e) => Err(format!("Unable to read config file: {}", e)),
        }
    }

    fn validate(c: ConfigFile) -> Result<ConfigFile, String> {
        validate_accounts(&c)?;
        validate_payee_normalizer_configs(&c)?;
        validate_payees(&c)?;
        validate_formats(&c)?;
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

impl Display for AccountConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Account: (id: {}, name: {})", self.id, self.name)
    }
}

impl Keyed<String> for AccountConfig {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

fn validate_accounts(c: &ConfigFile) -> Result<(), String> {
    for (a_id, a) in &c.accounts {
        if let Option::None = c.formats.get(&a.format_id) {
            return Err(format!("Format with id [{}] does not exist. The id is referenced from account with id [{}]",
                               a.format_id, a_id));
        }
    }
    Ok(())
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

fn validate_payee_normalizer_configs(c: &ConfigFile) -> Result<(), String> {
    for (a_id, a) in &c.accounts {
        for n in &a.payee_normalizers {
            if let MatcherType::Regex {ref regex_string} = n.normalizer_type {
                if let Err(e) = RegexBuilder::new(regex_string).build() {
                    return Err(format!("Invalid regex string provided to normalizer for payee [{}] in account [{}]: {}",
                                       n.payee_id, a_id, e));
                }
            }
            if let Option::None = a.payees.get(&n.payee_id) {
                return Err(format!("Invalid payee id [{}] used in a payee normalizer in account [{}]",
                                   n.payee_id, a_id));
            }
        }
    }
    Ok(())
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

impl Display for Payee {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Payee: (id: {}, name: {})", self.id, self.name)
    }
}

impl Keyed<String> for Payee {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

fn validate_payees(c: &ConfigFile) -> Result<(), String> {
    for (a_id, a) in &c.accounts {
        for (p_id, p) in &a.payees {
            if let Option::Some(ref category_ids) = p.category_ids {
                for c_id in category_ids {
                    if let Option::None = c.categories.get(c_id) {
                        return Err(format!("Category [{}] does not exist. Referenced from payee [{}] in account [{}].",
                                           c_id, p_id, a_id));
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Category {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
}

impl Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Category: (id: {}, name: {})", self.id, self.name)
    }
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

impl Display for FormatConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Format: (id: {}, name: {})", self.id, self.name)
    }
}

impl Keyed<String> for FormatConfig {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

fn validate_formats(c: &ConfigFile) -> Result<(), String> {
    for (f_id, f) in &c.formats {
        validate_date_time_config(f, &f.date_time_config)?;
        validate_amount_config(f, &f.amount_config)?;

        if !f.field_order.contains(&f.payee_config.field_name) {
            return Err(format!("Payee field name [{}] for format [{}] not included in field order.",
                               f.payee_config.field_name, f_id));
        }
        if let Option::Some(ref status_config) = f.status_config {
            if !f.field_order.contains(&status_config.field_name) {
                return Err(format!("Status field name [{}] for format [{}] not included in field order.",
                                   status_config.field_name, f_id));
            }
        }
        if let Option::Some(ref memo_config) = f.memo_config {
            if !f.field_order.contains(&memo_config.field_name) {
                return Err(format!("Status field name [{}] for format [{}] not included in field order.",
                                   memo_config.field_name, f_id));
            }
        }
        if let Option::Some(ref category_config) = f.category_config {
            if !f.field_order.contains(&category_config.field_name) {
                return Err(format!("Status field name [{}] for format [{}] not included in field order.",
                                   category_config.field_name, f_id));
            }
        }
    }
    Ok(())
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

fn validate_date_time_config(f: &FormatConfig, d: &DateTimeConfig) -> Result<(), String> {
    if !f.field_order.contains(&d.date_field) {
        return Err(format!("Date field name [{}] for format [{}] not included in field order.",
                           d.date_field, f.id));
    }
    if let Option::Some(ref time_field) = d.time_field {
        if !f.field_order.contains(time_field) {
            return Err(format!("Time field name [{}] for format [{}] not included in field order.",
                               time_field, f.id));
        }
    }
    Ok(())
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

fn validate_amount_config(f: &FormatConfig, a: &AmountConfig) -> Result<(), String> {
    match a.format {
        AmountFormat::SingleAmountField(ref c) => {
            if !f.field_order.contains(&c.field_name) {
                return Err(format!("Amount field name [{}] for format [{}] not included in field order.",
                                   c.field_name, f.id));
            }
        },
        AmountFormat::SeparateDebitCreditFields(ref c) => {
            if !f.field_order.contains(&c.debit_field) {
                return Err(format!("Debit field name [{}] for format [{}] not included in field order.",
                                   c.debit_field, f.id));
            }
            if !f.field_order.contains(&c.credit_field) {
                return Err(format!("Credit field name [{}] for format [{}] not included in field order.",
                                   c.credit_field, f.id));
            }
        },
        AmountFormat::TransactionTypeAndAmountFields(ref c) => {
            if !f.field_order.contains(&c.amount_field) {
                return Err(format!("Amount field name [{}] for format [{}] not included in field order.",
                                   c.amount_field, f.id));
            }
            if !f.field_order.contains(&c.transaction_type_field) {
                return Err(format!("Transaction type field name [{}] for format [{}] not included in field order.",
                                   c.transaction_type_field, f.id));
            }
        },
    }
    Ok(())
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
