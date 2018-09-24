use std::io;
use parser::{parse_csv_from_reader, create_csv_writer};
use std::collections::HashMap;
use transaction::{Transaction, TransactionStatus, TransactionType};
use config::{FormatConfig, AmountFormat};
use currency::{Currency};
use num::{Signed};
use std::ops::Neg;
use config::Config;

pub fn import_from_configurable_format(r: Box<io::Read>, f: &FormatConfig) -> Vec<Transaction> {
    let unmapped_transactions : Vec<HashMap<String, String>> = parse_csv_from_reader(r);

    let mut transactions = Vec::new();
    unmapped_transactions.into_iter().for_each(|x| {
        transactions.push(convert_to_transaction(x, f));
    });
    transactions
}

fn convert_to_transaction(unmapped: HashMap<String, String>, f: &FormatConfig) -> Transaction {
    let (amount, transaction_type) = get_amount_and_transaction_type(&unmapped, f);
    let (date_time_string, date_time_format) = get_date_time_and_format(&unmapped, f);
    Transaction::build(date_time_string,
                       date_time_format,
                       get_raw_payee_name(&unmapped, f),
                       get_category(&unmapped, f),
                       transaction_type,
                       amount,
                       get_transaction_status(&unmapped, f),
                       get_memo(&unmapped, f))
}

fn get_raw_payee_name(unmapped: &HashMap<String, String>, f: &FormatConfig) -> String {
    unmapped.get(&f.payee_config.field_name).and_then(|x| {
        Option::Some(x.to_owned())
    }).unwrap_or(Default::default())
}

const DEFAULT_TIME: &'static str = "00:00:00";
const DEFAULT_TIME_FORMAT: &'static str = "%T";
const DEFAULT_DATE_TIME_DELIMINATOR: &'static str = " ";

fn get_date_time_and_format(unmapped: &HashMap<String, String>, f: &FormatConfig) -> (String, String) {
    let date = unmapped.get(&f.date_time_config.date_field)
        .expect(&format!("No [{}] field available", f.date_time_config.date_field));
    let time = f.date_time_config.time_field.as_ref().and_then(|x| {
        unmapped.get(x)
    });

    let delim = f.date_time_config.deliminator.to_owned().unwrap_or(String::from(DEFAULT_DATE_TIME_DELIMINATOR));

    let (time, time_format) = if let Option::Some(time) = time {
        (time.to_owned(), f.date_time_config.time_format.to_owned().unwrap_or(String::from(DEFAULT_TIME_FORMAT)))
    } else {
        (String::from(DEFAULT_TIME), String::from(DEFAULT_TIME_FORMAT))
    };

    let format = format!("{}{}{}", f.date_time_config.date_format, delim, time_format);
    let date_time_string = format!("{}{}{}", date, delim, time);

    (date_time_string, format)
}

fn get_amount_and_transaction_type(unmapped: &HashMap<String, String>, f: &FormatConfig) -> (Currency, TransactionType) {
    match f.amount_config.format {
        AmountFormat::SingleAmountField(ref c) => {
            let amount = unmapped.get(&c.field_name).and_then(|x| {
                Option::Some(get_currency_from_str(x))
            }).unwrap_or(Default::default());

            let transaction_type = if amount.value().is_negative() {
                TransactionType::Debit
            } else {
                TransactionType::Credit
            };

            (amount, transaction_type)
        },
        AmountFormat::SeparateDebitCreditFields(ref c) => {
            if let Option::Some(amount) = unmapped.get(&c.debit_field).and_then(|x| {
                Option::Some(get_currency_from_str(x))
            }) {
                return (amount, TransactionType::Debit);
            }

            if let Option::Some(amount) = unmapped.get(&c.credit_field).and_then(|x| {
                Option::Some(get_currency_from_str(x))
            }) {
                return (amount, TransactionType::Credit);
            }

            panic!("No transaction amount detected");
        },
        AmountFormat::TransactionTypeAndAmountFields(ref c) => {
            let amount = unmapped.get(&c.amount_field).and_then(|x| {
                Option::Some(get_currency_from_str(x))
            }).unwrap_or(Default::default());
            let transaction_type = unmapped.get(&c.transaction_type_field).and_then(|x| {
                if x == &c.credit_string {
                    return Option::Some(TransactionType::Credit);
                } else if x == &c.debit_string {
                    return Option::Some(TransactionType::Debit);
                }
                Option::None
            }).unwrap_or(TransactionType::Debit);
            (amount, transaction_type)
        }
    }
}

fn get_currency_from_str(s: &str) -> Currency {
    Currency::from_str(s).expect(&format!("Unable to parse amount into a valid currency: {}", s))
}

fn get_transaction_status(unmapped: &HashMap<String, String>, f: &FormatConfig) -> TransactionStatus {
    f.status_config.as_ref().and_then(|c| {
        unmapped.get(&c.field_name).and_then(|x| {
            if x == &c.cleared_string {
                return Option::Some(TransactionStatus::Cleared);
            } else if x == &c.pending_string {
                return Option::Some(TransactionStatus::Pending);
            }
            Option::None
        })
    }).unwrap_or(TransactionStatus::Cleared)
}

fn get_memo(unmapped: &HashMap<String, String>, f: &FormatConfig) -> Option<String> {
    f.memo_config.as_ref().and_then(|c| {
        unmapped.get(&c.field_name).and_then(|x| {
            Option::Some(x.to_owned())
        })
    })
}

fn get_category(unmapped: &HashMap<String, String>, f: &FormatConfig) -> Option<String> {
    f.category_config.as_ref().and_then(|c| {
        unmapped.get(&c.field_name).and_then(|x| {
            Option::Some(x.to_owned())
        })
    })
}

/// Assumes CSV
pub fn export_to_configurable_format(w: Box<io::Write>, c: &Config, f: &FormatConfig, transactions: Vec<Transaction>) {
    let mut w = create_csv_writer(c.include_header(), w);
    if c.include_header() {
        w.write_record(&f.field_order).expect("Error writing record");
    }
    transactions.iter().for_each(|t| {
        w.write_record(convert_to_configurable_format(f, t)).expect("Error writing record");
    });
}

fn convert_to_configurable_format(f: &FormatConfig, t: &Transaction) -> Vec<String> {
    let mut fields = HashMap::new();

    // Date
    fields.insert(f.date_time_config.date_field.to_owned(),
                  t.date().format(&f.date_time_config.date_format).to_string());
    // Time
    if let Option::Some(ref time_field) = f.date_time_config.time_field {
        fields.insert(time_field.to_owned(),
                      t.date().format(&f.date_time_config.time_format.as_ref()
                          .unwrap_or(&String::from(DEFAULT_TIME_FORMAT))).to_string());
    }

    // Payee
    fields.insert(f.payee_config.field_name.to_owned(), t.payee().to_owned());

    // Category
    if let Option::Some(ref c) = f.category_config {
        fields.insert(c.field_name.to_owned(), t.category.to_owned().unwrap_or(Default::default()));
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
        fields.insert(c.field_name.to_owned(), t.memo.as_ref().unwrap_or(&Default::default()).to_owned());
    }

    // Put the fields in the correct order
    let mut r = Vec::new();
    for field in &f.field_order {
        r.push(fields.remove(field).unwrap_or(Default::default()));
    }
    r
}

/// Get the amount fields for a transaction. Returns a list of mappings from 'field name' -> 'field value'
fn get_amount_fields(f: &FormatConfig, t: &Transaction) -> Vec<(String, String)> {
    let mut r = Vec::new();
    match f.amount_config.format {
        AmountFormat::SingleAmountField(ref c) => {
            let amount = match t.transaction_type {
                TransactionType::Debit => currency_to_string_without_delim(&t.amount.to_owned().neg()),
                TransactionType::Credit => currency_to_string_without_delim(&t.amount),
            };
            r.push((c.field_name.to_owned(), amount));
        },
        AmountFormat::SeparateDebitCreditFields(ref c) => {
            let (debit_amount, credit_amount) = match t.transaction_type {
                TransactionType::Debit => (Option::Some(&t.amount), Option::None),
                TransactionType::Credit => (Option::None, Option::Some(&t.amount)),
            };
            r.push((c.debit_field.to_owned(), debit_amount.map_or(Default::default(), |x| {
                currency_to_string_without_delim(x)
            })));
            r.push((c.credit_field.to_owned(), credit_amount.map_or(Default::default(), |x| {
                currency_to_string_without_delim(x)
            })));
        },
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
            r.push((c.amount_field.to_owned(), currency_to_string_without_delim(&amount)));
        },
    }
    r
}

// For some reason, the Currency type prepends a ',' to values in the hundreds, so just remove
// all ',' from the string generated by Currency to avoid such silliness.
fn currency_to_string_without_delim(c: &Currency) -> String {
    let s = c.to_string();
    s.replace(",", "")
}