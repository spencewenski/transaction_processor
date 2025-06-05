use crate::config::Config;
use crate::transaction::payee::PayeeNormalizer;
use crate::util::currency_to_string_without_delim;
use chrono::prelude::*;
use currency::Currency;
use num::Signed;
use std::ops::Neg;
use typed_builder::TypedBuilder;

pub mod payee;
pub mod transaction_io;

#[derive(Debug, TypedBuilder)]
pub struct Transaction {
    date: NaiveDateTime,
    #[builder(setter(transform = |value: String| InputCleaner::clean(value) ))]
    raw_payee_name: String,
    #[builder(default)]
    normalized_payee_id: Option<String>,
    #[builder(default)]
    normalized_payee_name: Option<String>,
    #[builder(default, setter(transform = |value: Option<String>| value.map(InputCleaner::clean )))]
    category: Option<String>,
    transaction_type: TransactionType,
    // Non-negative
    #[builder(setter(transform = |value: Currency| get_currency_absolute_value(value) ))]
    amount: Currency,
    status: TransactionStatus,
    #[builder(default, setter(transform = |value: Option<String>| value.map(InputCleaner::clean )))]
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
    pub fn normalize_payee(&mut self, config: &Config) {
        self.normalized_payee_id =
            PayeeNormalizer::normalized_payee_id(config, &self.raw_payee_name);
        self.normalized_payee_name = self
            .normalized_payee_id
            .as_ref()
            .and_then(|p| config.account().payees.get(p))
            .map(|x| x.name.to_owned());
    }

    pub fn categorize(&mut self, config: &Config) {
        self.category = PayeeNormalizer::category_for_transaction(config, self);
        if self.category.is_none() {
            println!(
                "Transaction was not categorized: [payee: {}], [amount: {}], [type: {:?}], [date: {}]",
                self.payee(),
                currency_to_string_without_delim(&self.amount),
                self.transaction_type,
                self.date
            );
        }
    }

    pub fn date(&self) -> &NaiveDateTime {
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
    if c.value().is_negative() { c.neg() } else { c }
}

struct InputCleaner;
trait Clean<T> {
    fn clean(s: T) -> T;
}

impl Clean<String> for InputCleaner {
    fn clean(s: String) -> String {
        s.trim().replace('\n', " ")
    }
}

impl Clean<Option<String>> for InputCleaner {
    fn clean(s: Option<String>) -> Option<String> {
        s.map(Self::clean)
    }
}
