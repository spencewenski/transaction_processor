use chrono::prelude::*;
use transaction::payee::PayeeNormalizer;
use config::Config;
use currency::Currency;
use num::{Signed};
use std::ops::Neg;
use util::{currency_to_string_without_delim};

pub mod payee;
pub mod transaction_io;

#[derive(Debug)]
pub struct Transaction {
    date: DateTime<Utc>,
    raw_payee_name: String,
    normalized_payee_id: Option<String>,
    normalized_payee_name: Option<String>,
    category: Option<String>,
    transaction_type: TransactionType,
    // Non-negative
    amount: Currency,
    status: TransactionStatus,
    memo: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
enum TransactionType {
    Debit,
    Credit,
}

#[derive(Debug, Eq, PartialEq)]
enum TransactionStatus {
    Pending,
    Cleared,
}

impl Transaction {
    fn build(date: String,
             date_format: String,
             payee: String,
             category: Option<String>,
             transaction_type: TransactionType,
             amount: Currency,
             status: TransactionStatus,
             memo: Option<String>) -> Result<Transaction, String> {
        let date = match Utc.datetime_from_str(&date, &date_format) {
            Ok(date) => Ok(date),
            Err(e) => Err(format!("Unable to parse date string [{}] using date format string [{}]; error: {}", date, date_format, e)),
        }?;
        Ok(Transaction {
            date,
            raw_payee_name: InputCleaner::clean(payee),
            normalized_payee_id: Option::None,
            normalized_payee_name: Option::None,
            category: InputCleaner::clean(category),
            transaction_type,
            amount: get_currency_absolute_value(amount),
            status,
            memo: InputCleaner::clean(memo),
        })
    }

    pub fn normalize_payee(&mut self, config: &Config) {
        self.normalized_payee_id = PayeeNormalizer::normalized_payee_id(config, &self.raw_payee_name);
        self.normalized_payee_name = self.normalized_payee_id.as_ref().and_then(|p| {
            config.account().payees.get(p)
        }).and_then(|x| {
            Option::Some(x.name.to_owned())
        });
    }

    pub fn categorize(&mut self, config: &Config) {
        self.category = PayeeNormalizer::category_for_transaction(config, &self);
        if let Option::None = self.category {
            println!("Transaction was not categorized: [payee: {}], [amount: {}], [date: {}]",
                     self.payee(), currency_to_string_without_delim(&self.amount), self.date);
        }
    }

    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
    }

    // Get the name of the payee for this transaction. Either the raw payee name, or the
    // normalized name if it has been normalized.
    pub fn payee(&self) -> &str {
        if let Option::Some(ref p) = self.normalized_payee_name {
            p
        } else {
            &self.raw_payee_name
        }
    }
}

fn get_currency_absolute_value(c: Currency) -> Currency {
    if c.value().is_negative() {
        c.neg()
    } else {
        c
    }
}

struct InputCleaner;
trait Clean<T> {
    fn clean(s: T) -> T;
}

impl Clean<String> for InputCleaner {
    fn clean(s: String) -> String {
        s.trim().replace("\n", " ")
    }
}

impl Clean<Option<String>> for InputCleaner {
    fn clean(s: Option<String>) -> Option<String> {
        match s {
            Option::Some(s) => Option::Some(Self::clean(s)),
            _ => Option::None,
        }
    }
}
