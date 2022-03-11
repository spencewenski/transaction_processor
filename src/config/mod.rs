use anyhow::anyhow;
use clap::Parser;
use config::arguments::Arguments;
use parser::{default_false, default_true, deserialize_keyed_items, Keyed};
use regex::RegexBuilder;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;
use util;

mod arguments;

pub struct Config {
    args: Arguments,
    account_config_file: AccountConfigFile,
    categories_config_file: CategoriesConfigFile,
    src_format_config_file: FormatConfigFile,
    dst_format_config_file: FormatConfigFile,
}

impl Config {
    pub fn new_and_parse_args() -> anyhow::Result<Config> {
        let args: Arguments = Arguments::parse();
        let account_config_file = AccountConfigFile::from_file(&args.account_config_file)?;
        let categories_config_file = CategoriesConfigFile::from_file(&args.categories_config_file)?;
        let src_format_config_file = FormatConfigFile::from_file(&args.src_format_config_file)?;
        let dst_format_config_file = FormatConfigFile::from_file(&args.dst_format_config_file)?;

        let config = Config {
            args,
            account_config_file,
            categories_config_file,
            src_format_config_file,
            dst_format_config_file,
        };

        validate_configs(&config)?;

        Ok(config)
    }

    pub fn account(&self) -> &AccountConfigFile {
        // We validated the input, so this should never return Option::None
        &self.account_config_file
    }

    pub fn src_format(&self) -> &FormatConfigFile {
        // We validated the input, so this should never return Option::None
        &self.src_format_config_file
    }

    pub fn dst_format(&self) -> &FormatConfigFile {
        // We validated the input, so this should never return Option::None
        &self.dst_format_config_file
    }

    pub fn src_file(&self) -> Option<&PathBuf> {
        self.args.src_file.as_ref()
    }

    pub fn dst_file(&self) -> Option<&PathBuf> {
        self.args.dst_file.as_ref()
    }

    pub fn category(&self, category_id: &str) -> Option<&Category> {
        self.categories_config_file.categories.get(category_id)
    }

    pub fn sort_order(&self) -> Option<SortOrder> {
        if self.args.sort_order.is_some() {
            return self.args.sort_order.clone();
        }

        self.dst_format()
            .sort
            .as_ref()
            .map(|sort| sort.order.clone())
            .or(Some(SortOrder::Ascending))
    }

    pub fn sort_by(&self) -> Option<SortBy> {
        if self.args.sort_by.is_some() {
            return self.args.sort_by.clone();
        }

        self.dst_format()
            .sort
            .as_ref()
            .map(|sort| sort.sort_by.clone())
            .or(Some(SortBy::Date))
    }

    /// Whether to include the header in CSV output
    pub fn include_header(&self) -> bool {
        self.args
            .include_header
            .or(self.dst_format().include_header)
            .unwrap_or(false)
    }

    pub fn ignore_pending(&self) -> bool {
        self.args
            .ignore_pending
            .or(self.account().ignore_pending)
            .unwrap_or(false)
    }

    pub fn skip_prompts(&self) -> bool {
        self.args
            .skip_prompts
            .or(self.account().skip_prompts)
            .unwrap_or(false)
    }
}

fn validate_configs(config: &Config) -> anyhow::Result<()> {
    validate_account_config(config)?;
    validate_payee_normalizer_configs(config)?;
    validate_payees(config)?;
    validate_format(&config.src_format_config_file)?;
    validate_format(&config.dst_format_config_file)?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CategoriesConfigFile {
    #[serde(rename = "categories", deserialize_with = "deserialize_keyed_items")]
    pub categories: HashMap<String, Category>,
}

impl CategoriesConfigFile {
    fn from_file(filename: &PathBuf) -> anyhow::Result<CategoriesConfigFile> {
        let r = util::reader_from_file_name(filename)?;
        let config_file: CategoriesConfigFile = serde_json::from_reader(r)?;
        Ok(config_file)
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountConfigFile {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "formatId")]
    pub format_id: String,
    #[serde(rename = "ignorePending")]
    ignore_pending: Option<bool>,
    #[serde(rename = "skipPrompts")]
    skip_prompts: Option<bool>,
    #[serde(rename = "payees", deserialize_with = "deserialize_keyed_items")]
    pub payees: HashMap<String, Payee>,
}

impl AccountConfigFile {
    fn from_file(filename: &PathBuf) -> anyhow::Result<AccountConfigFile> {
        let r = util::reader_from_file_name(filename)?;
        let config_file: AccountConfigFile = serde_json::from_reader(r)?;
        Ok(config_file)
    }
}

impl Display for AccountConfigFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Account: (id: {}, name: {})", self.id, self.name)
    }
}

impl Keyed<String> for AccountConfigFile {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

fn validate_account_config(config: &Config) -> anyhow::Result<()> {
    if config.account_config_file.format_id != config.src_format_config_file.id {
        Err(anyhow!("Format ID [{}] for account [{}] is different from the ID of the provided source format file [{}].",
            config.account_config_file.format_id,
            config.account_config_file.name,
            config.args.src_format_config_file.to_str().unwrap_or("Invalid file name")))
    } else {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct PayeeNormalizerConfig {
    #[serde(rename = "matcher")]
    #[serde(flatten)]
    pub normalizer_type: MatcherType,
    #[serde(rename = "ignoreCase", default = "default_true")]
    pub ignore_case: bool,
}

fn validate_payee_normalizer_configs(config: &Config) -> anyhow::Result<()> {
    for (p_id, payee) in &config.account_config_file.payees {
        for normalizer in &payee.normalizers {
            if let MatcherType::Regex { ref regex_string } = normalizer.normalizer_type {
                if let Err(e) = RegexBuilder::new(regex_string).build() {
                    return Err(anyhow!(
                        "Invalid regex string provided to normalizer for payee [{}]: {}",
                        p_id,
                        e
                    ));
                }
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
    #[serde(rename = "normalizers")]
    pub normalizers: Vec<PayeeNormalizerConfig>,
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

fn validate_payees(config: &Config) -> anyhow::Result<()> {
    for (p_id, p) in &config.account_config_file.payees {
        if let Option::Some(ref category_ids) = p.category_ids {
            for c_id in category_ids {
                if let Option::None = config.categories_config_file.categories.get(c_id) {
                    return Err(anyhow!(
                        "Category [{}] does not exist. Referenced from payee [{}].",
                        c_id,
                        p_id
                    ));
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

#[derive(Debug, Deserialize, PartialEq, Clone, clap::ArgEnum)]
pub enum SortBy {
    #[serde(rename = "date")]
    Date,
}

#[derive(Debug, Deserialize, PartialEq, Clone, clap::ArgEnum)]
pub enum SortOrder {
    #[serde(rename = "ascending")]
    Ascending,
    #[serde(rename = "descending")]
    Descending,
}

// todo: should FormatConfig go in the 'formats' module?
#[derive(Debug, Deserialize)]
pub struct FormatConfigFile {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "includeHeader")]
    include_header: Option<bool>,
    #[serde(rename = "sort")]
    sort: Option<Sort>,
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
    pub category_config: Option<CategoryConfig>,
}

impl FormatConfigFile {
    fn from_file(filename: &PathBuf) -> anyhow::Result<FormatConfigFile> {
        let r = util::reader_from_file_name(filename)?;
        let config_file: FormatConfigFile = serde_json::from_reader(r)?;
        Ok(config_file)
    }
}

impl Display for FormatConfigFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Format: (id: {}, name: {})", self.id, self.name)
    }
}

impl Keyed<String> for FormatConfigFile {
    fn key(&self) -> String {
        self.id.to_owned()
    }
}

fn validate_format(format_config: &FormatConfigFile) -> anyhow::Result<()> {
    validate_date_time_config(format_config, &format_config.date_time_config)?;
    validate_amount_config(format_config, &format_config.amount_config)?;

    if !format_config
        .field_order
        .contains(&format_config.payee_config.field_name)
    {
        return Err(anyhow!(
            "Payee field name [{}] for format [{}] not included in field order.",
            format_config.payee_config.field_name,
            format_config.id
        ));
    }
    if let Option::Some(ref status_config) = format_config.status_config {
        if !format_config
            .field_order
            .contains(&status_config.field_name)
        {
            return Err(anyhow!(
                "Status field name [{}] for format [{}] not included in field order.",
                status_config.field_name,
                format_config.id
            ));
        }
    }
    if let Option::Some(ref memo_config) = format_config.memo_config {
        if !format_config.field_order.contains(&memo_config.field_name) {
            return Err(anyhow!(
                "Status field name [{}] for format [{}] not included in field order.",
                memo_config.field_name,
                format_config.id
            ));
        }
    }
    if let Option::Some(ref category_config) = format_config.category_config {
        if !format_config
            .field_order
            .contains(&category_config.field_name)
        {
            return Err(anyhow!(
                "Status field name [{}] for format [{}] not included in field order.",
                category_config.field_name,
                format_config.id
            ));
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

fn validate_date_time_config(f: &FormatConfigFile, d: &DateTimeConfig) -> anyhow::Result<()> {
    if !f.field_order.contains(&d.date_field) {
        return Err(anyhow!(
            "Date field name [{}] for format [{}] not included in field order.",
            d.date_field,
            f.id
        ));
    }
    if let Option::Some(ref time_field) = d.time_field {
        if !f.field_order.contains(time_field) {
            return Err(anyhow!(
                "Time field name [{}] for format [{}] not included in field order.",
                time_field,
                f.id
            ));
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

fn validate_amount_config(f: &FormatConfigFile, a: &AmountConfig) -> anyhow::Result<()> {
    match a.format {
        AmountFormat::SingleAmountField(ref c) => {
            if !f.field_order.contains(&c.field_name) {
                return Err(anyhow!(
                    "Amount field name [{}] for format [{}] not included in field order.",
                    c.field_name,
                    f.id
                ));
            }
        }
        AmountFormat::SeparateDebitCreditFields(ref c) => {
            if !f.field_order.contains(&c.debit_field) {
                return Err(anyhow!(
                    "Debit field name [{}] for format [{}] not included in field order.",
                    c.debit_field,
                    f.id
                ));
            }
            if !f.field_order.contains(&c.credit_field) {
                return Err(anyhow!(
                    "Credit field name [{}] for format [{}] not included in field order.",
                    c.credit_field,
                    f.id
                ));
            }
        }
        AmountFormat::TransactionTypeAndAmountFields(ref c) => {
            if !f.field_order.contains(&c.amount_field) {
                return Err(anyhow!(
                    "Amount field name [{}] for format [{}] not included in field order.",
                    c.amount_field,
                    f.id
                ));
            }
            if !f.field_order.contains(&c.transaction_type_field) {
                return Err(anyhow!(
                    "Transaction type field name [{}] for format [{}] not included in field order.",
                    c.transaction_type_field,
                    f.id
                ));
            }
        }
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
    #[serde(rename = "debitIsNegative", default = "default_false")]
    pub debit_is_negative: bool,
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
