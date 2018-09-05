use chrono::prelude::*;
use transaction::payee::PayeeNormalizer;
use arguments::Arguments;
use config::Config;

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
             date_format: String,
             payee: String,
             category: Option<String>,
             transaction_type: TransactionType,
             amount: String,
             status: TransactionStatus,
             memo: Option<String>) -> Transaction {
        Transaction {
            date: Utc.datetime_from_str(&date, &date_format).unwrap(),
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

    pub fn normalize_payee(&mut self, account_id: &str, config: &Config) {
        self.normalized_payee_id = PayeeNormalizer::normalized_payee_id(config, account_id, &self.raw_payee_name);
        self.normalized_payee_name = config.accounts.get(account_id).and_then(|a| {
            self.normalized_payee_id.as_ref().and_then(|p| {
                a.payees.get(p)
            })
        }).and_then(|x| {
            Option::Some(x.name.to_owned())
        });
    }

    pub fn categorize(&mut self, args: &Arguments, account_id: &str, config: &Config) {
        self.category = PayeeNormalizer::category_for_transaction(args, config, account_id, &self);
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
