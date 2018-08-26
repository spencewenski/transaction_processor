use chrono::prelude::*;
use transaction::payee::PayeeNormalizer;
use arguments::Arguments;

pub mod payee;
pub mod formats;
pub mod transaction_io;
pub mod account;

#[derive(Debug)]
pub struct Transaction {
    date: DateTime<Utc>,
    raw_payee_name: String,
    normalized_payee_id: Option<String>,
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
            normalized_payee_id: Option::None,
            normalized_payee_name: Option::None,
            category: InputCleaner::clean(category),
            transaction_type,
            amount: InputCleaner::clean(amount),
            status,
            memo: InputCleaner::clean(memo),
        }
    }

    pub fn normalize_payee(&mut self, account_id: Option<String>, normalizer: &PayeeNormalizer) {
        self.normalized_payee_id = normalizer.normalized_payee_id(account_id.to_owned(), &self.raw_payee_name);
        if let (Option::Some(a), Option::Some(p)) = (&account_id, &self.normalized_payee_id) {
            let p = normalizer.payee(&a, &p);
            if let Option::Some(p) = p {
                self.normalized_payee_name = Option::Some(p.name.to_owned())
            }
        }
    }

    pub fn categorize(&mut self, args: &Arguments, account_id: Option<String>, normalizer: &PayeeNormalizer) {
        self.category = normalizer.category_for_transaction(args, account_id, &self);
        if let Option::None = self.category {
            println!("Transaction was not categorized: [payee: {}], [amount: {}], [date: {}]", self.payee(), self.amount, self.date);
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
