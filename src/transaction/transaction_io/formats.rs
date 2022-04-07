use anyhow::anyhow;
use config::Config;
use config::{AmountFormat, FormatConfigFile};
use csv::Writer;
use currency::Currency;
use num::Signed;
use parser::{create_csv_writer, parse_csv_from_reader};
use std::collections::HashMap;
use std::io;
use std::ops::Neg;
use transaction::{Transaction, TransactionStatus, TransactionType};
use util::{currency_to_string_without_delim, get_optional_string};

pub fn import_from_configurable_format(
    r: Box<dyn io::Read>,
    f: &FormatConfigFile,
) -> anyhow::Result<Vec<Transaction>> {
    let unmapped_transactions: Vec<HashMap<String, String>> = parse_csv_from_reader(r)?;

    let mut transactions = Vec::new();
    for unmapped in unmapped_transactions {
        let t = convert_to_transaction(unmapped, f)?;
        transactions.push(t);
    }
    Ok(transactions)
}

fn convert_to_transaction(
    unmapped: HashMap<String, String>,
    f: &FormatConfigFile,
) -> anyhow::Result<Transaction> {
    let (amount, transaction_type) = get_amount_and_transaction_type(&unmapped, f)?;
    let (date_time_string, date_time_format) = get_date_time_and_format(&unmapped, f)?;
    Transaction::build(
        date_time_string,
        date_time_format,
        get_raw_payee_name(&unmapped, f)?,
        get_category(&unmapped, f),
        transaction_type,
        amount,
        get_transaction_status(&unmapped, f)?,
        get_memo(&unmapped, f),
    )
}

fn get_raw_payee_name(
    unmapped: &HashMap<String, String>,
    f: &FormatConfigFile,
) -> anyhow::Result<String> {
    match unmapped.get(&f.payee_config.field_name) {
        Option::Some(p) => Ok(p.to_owned()),
        _ => Err(anyhow!(
            "Payee field [{}] does not exist.",
            f.payee_config.field_name
        )),
    }
}

const DEFAULT_TIME: &str = "00:00:00";
const DEFAULT_TIME_FORMAT: &str = "%T";
const DEFAULT_DATE_TIME_DELIMINATOR: &str = " ";

fn get_date_time_and_format(
    unmapped: &HashMap<String, String>,
    f: &FormatConfigFile,
) -> anyhow::Result<(String, String)> {
    let date = match unmapped.get(&f.date_time_config.date_field) {
        Option::Some(d) => Ok(d),
        _ => Err(anyhow!(
            "Date field [{}] does not exist.",
            f.date_time_config.date_field
        )),
    }?;
    let time = f
        .date_time_config
        .time_field
        .as_ref()
        .and_then(|x| unmapped.get(x));

    let delim = f
        .date_time_config
        .deliminator
        .to_owned()
        .unwrap_or_else(|| String::from(DEFAULT_DATE_TIME_DELIMINATOR));

    let (time, time_format) = if let Option::Some(time) = time {
        (
            time.to_owned(),
            f.date_time_config
                .time_format
                .to_owned()
                .unwrap_or_else(|| String::from(DEFAULT_TIME_FORMAT)),
        )
    } else {
        (
            String::from(DEFAULT_TIME),
            String::from(DEFAULT_TIME_FORMAT),
        )
    };

    let format = format!("{}{}{}", f.date_time_config.date_format, delim, time_format);
    let date_time_string = format!("{}{}{}", date, delim, time);

    Ok((date_time_string, format))
}

fn get_amount_and_transaction_type(
    unmapped: &HashMap<String, String>,
    f: &FormatConfigFile,
) -> anyhow::Result<(Currency, TransactionType)> {
    match f.amount_config.format {
        AmountFormat::SingleAmountField(ref c) => {
            let amount = unmapped
                .get(&c.field_name)
                .and_then(|a| get_currency_from_str(a));
            let amount = match amount {
                Option::Some(a) => a,
                _ => Err(anyhow!("Amount field [{}] does not exist.", &c.field_name)),
            }?;
            let transaction_type = if amount.value().is_negative() {
                if c.debit_is_negative {
                    TransactionType::Debit
                } else {
                    TransactionType::Credit
                }
            } else {
                if c.debit_is_negative {
                    TransactionType::Credit
                } else {
                    TransactionType::Debit
                }
            };

            Ok((amount, transaction_type))
        }
        AmountFormat::SeparateDebitCreditFields(ref c) => {
            if let Option::Some(amount) = unmapped
                .get(&c.debit_field)
                .and_then(|x| get_currency_from_str(x))
            {
                return Ok((amount?, TransactionType::Debit));
            }

            if let Option::Some(amount) = unmapped
                .get(&c.credit_field)
                .and_then(|x| get_currency_from_str(x))
            {
                return Ok((amount?, TransactionType::Credit));
            }

            Err(anyhow!(
                "Neither the debit field [{}] nor the credit field [{}] exists.",
                c.debit_field,
                c.credit_field
            ))
        }
        AmountFormat::TransactionTypeAndAmountFields(ref c) => {
            let amount = unmapped
                .get(&c.amount_field)
                .and_then(|a| get_currency_from_str(a));
            let amount = match amount {
                Option::Some(a) => a,
                _ => Err(anyhow!("Amount field [{}] does not exist.", c.amount_field)),
            }?;
            let transaction_type = match unmapped.get(&c.transaction_type_field) {
                Option::Some(t) => {
                    if t == &c.credit_string {
                        Ok(TransactionType::Credit)
                    } else if t == &c.debit_string {
                        Ok(TransactionType::Debit)
                    } else {
                        Err(anyhow!("String [{}] matches neither the credit string [{}] nor the debit string [{}]",
                                    t, c.credit_string, c.debit_string))
                    }
                }
                _ => Err(anyhow!(
                    "Transaction type field [{}] does not exist.",
                    c.transaction_type_field
                )),
            }?;
            Ok((amount, transaction_type))
        }
    }
}

fn get_currency_from_str(s: &str) -> Option<anyhow::Result<Currency>> {
    get_optional_string(s).map(|s| {
        Currency::from_str(&s).map_err(|e| {
            anyhow!(
                "Unable to parse amount [{}] into a valid currency: {}",
                s,
                e
            )
        })
    })
}

fn get_transaction_status(
    unmapped: &HashMap<String, String>,
    f: &FormatConfigFile,
) -> anyhow::Result<TransactionStatus> {
    if let Option::Some(ref c) = f.status_config {
        return match unmapped.get(&c.field_name) {
            Option::Some(s) => {
                if s == &c.cleared_string {
                    Ok(TransactionStatus::Cleared)
                } else if s == &c.pending_string {
                    Ok(TransactionStatus::Pending)
                } else {
                    Err(anyhow!("String [{}] matches neither the cleard string [{}] nor the pending string [{}]",
                                s, c.cleared_string, c.pending_string))
                }
            }
            _ => Err(anyhow!(
                "Transaction status field [{}] does not exist.",
                c.field_name
            )),
        };
    }
    Ok(TransactionStatus::Cleared)
}

fn get_memo(unmapped: &HashMap<String, String>, f: &FormatConfigFile) -> Option<String> {
    f.memo_config
        .as_ref()
        .and_then(|c| unmapped.get(&c.field_name))
        .map(|x| x.to_owned())
}

fn get_category(unmapped: &HashMap<String, String>, f: &FormatConfigFile) -> Option<String> {
    f.category_config
        .as_ref()
        .and_then(|c| unmapped.get(&c.field_name))
        .map(|x| x.to_owned())
}

/// Assumes CSV
pub fn export_to_configurable_format(
    w: Box<dyn io::Write>,
    c: &Config,
    f: &FormatConfigFile,
    transactions: Vec<Transaction>,
) -> anyhow::Result<()> {
    let mut w = create_csv_writer(c.include_header(), w);
    if c.include_header() {
        write_record(&mut w, &f.field_order)?;
    }
    for t in &transactions {
        write_record(&mut w, &convert_to_configurable_format(f, t))?;
    }
    Ok(())
}

fn write_record(w: &mut Writer<Box<dyn io::Write>>, r: &[String]) -> anyhow::Result<()> {
    if let Err(e) = w.write_record(r) {
        return Err(anyhow!(
            "An error occurred while writing to the destination: {}",
            e
        ));
    }
    Ok(())
}

fn convert_to_configurable_format(f: &FormatConfigFile, t: &Transaction) -> Vec<String> {
    let mut fields = HashMap::new();

    // Date
    fields.insert(
        f.date_time_config.date_field.to_owned(),
        t.date().format(&f.date_time_config.date_format).to_string(),
    );
    // Time
    if let Option::Some(ref time_field) = f.date_time_config.time_field {
        fields.insert(
            time_field.to_owned(),
            t.date()
                .format(
                    f.date_time_config
                        .time_format
                        .as_ref()
                        .unwrap_or(&String::from(DEFAULT_TIME_FORMAT)),
                )
                .to_string(),
        );
    }

    // Payee
    fields.insert(f.payee_config.field_name.to_owned(), t.payee().to_owned());

    // Category
    if let Option::Some(ref c) = f.category_config {
        fields.insert(
            c.field_name.to_owned(),
            t.category.to_owned().unwrap_or_default(),
        );
    }

    // Transaction status
    if let Option::Some(ref c) = f.status_config {
        let status = match t.status {
            TransactionStatus::Pending => c.pending_string.to_owned(),
            TransactionStatus::Cleared => c.cleared_string.to_owned(),
        };
        fields.insert(c.field_name.to_owned(), status);
    }

    // Amount and transaction type
    let amount_fields = get_amount_fields(f, t);
    for (field_name, amount) in amount_fields {
        fields.insert(field_name, amount);
    }

    // Memo
    if let Option::Some(ref c) = f.memo_config {
        fields.insert(
            c.field_name.to_owned(),
            t.memo.as_ref().unwrap_or(&Default::default()).to_owned(),
        );
    }

    // Put the fields in the correct order
    let mut r = Vec::new();
    for field in &f.field_order {
        r.push(fields.remove(field).unwrap_or_default());
    }
    r
}

/// Get the amount fields for a transaction. Returns a list of mappings from 'field name' -> 'field value'
fn get_amount_fields(f: &FormatConfigFile, t: &Transaction) -> Vec<(String, String)> {
    let mut r = Vec::new();
    match f.amount_config.format {
        AmountFormat::SingleAmountField(ref c) => {
            let amount = match t.transaction_type {
                TransactionType::Debit => {
                    currency_to_string_without_delim(&t.amount.to_owned().neg())
                }
                TransactionType::Credit => currency_to_string_without_delim(&t.amount),
            };
            r.push((c.field_name.to_owned(), amount));
        }
        AmountFormat::SeparateDebitCreditFields(ref c) => {
            let (debit_amount, credit_amount) = match t.transaction_type {
                TransactionType::Debit => (Option::Some(&t.amount), Option::None),
                TransactionType::Credit => (Option::None, Option::Some(&t.amount)),
            };
            r.push((
                c.debit_field.to_owned(),
                debit_amount.map_or(Default::default(), currency_to_string_without_delim),
            ));
            r.push((
                c.credit_field.to_owned(),
                credit_amount.map_or(Default::default(), currency_to_string_without_delim),
            ));
        }
        AmountFormat::TransactionTypeAndAmountFields(ref c) => {
            let transaction_type = match t.transaction_type {
                TransactionType::Debit => c.debit_string.to_owned(),
                TransactionType::Credit => c.credit_string.to_owned(),
            };
            let amount = if t.transaction_type == TransactionType::Debit && c.include_debit_sign {
                t.amount.to_owned().neg()
            } else {
                t.amount.to_owned()
            };
            r.push((c.transaction_type_field.to_owned(), transaction_type));
            r.push((
                c.amount_field.to_owned(),
                currency_to_string_without_delim(&amount),
            ));
        }
    }
    r
}
