use chrono::prelude::*;
use transaction::payee::PayeeNormalizer;

pub mod payee;
pub mod formats;
pub mod transaction_io;
pub mod account;

#[derive(Debug)]
pub struct Transaction {
    date: DateTime<Utc>,
    raw_payee_name: String,
    normalized_payee_name: Option<String>,
    category: Option<String>,
    transaction_type: TransactionType,
    amount: String,
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
             date_format: &str,
             payee: String,
             category: Option<String>,
             transaction_type: TransactionType,
             amount: String,
             status: TransactionStatus,
             memo: Option<String>) -> Transaction {
        Transaction {
            date: Utc.datetime_from_str(&date, date_format).unwrap(),
            raw_payee_name: InputCleaner::clean(payee),
            normalized_payee_name: Option::None,
            category: InputCleaner::clean(category),
            transaction_type,
            amount: InputCleaner::clean(amount),
            status,
            memo: InputCleaner::clean(memo),
        }
    }

    pub fn normalize_payee(&mut self, normalizer: &PayeeNormalizer) {
        self.normalized_payee_name = Option::Some(normalizer.normalize_str(&self.raw_payee_name));
    }

    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
    }

    // Get the name of the payee for this transaction. Either the raw payee name, or the
    // normalized name if it has been normalized.
    pub fn payee(&self) -> &str {
        if let Option::Some(ref s) = self.normalized_payee_name {
            return s;
        } else {
            return &self.raw_payee_name;
        }
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
