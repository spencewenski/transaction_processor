use chrono::prelude::*;

pub mod payee;
pub mod formats;

#[derive(Debug)]
pub struct Transaction {
    date: DateTime<Utc>,
    payee: payee::PayeeType,
    category: Option<String>,
    transaction_type: TransactionType,
    amount: String,
    status: TransactionStatus,
    memo: Option<String>,
}

#[derive(Debug)]
enum TransactionType {
    Debit,
    Credit,
}

#[derive(Debug)]
enum TransactionStatus {
    Pending,
    Cleared,
}

impl Transaction {
    fn build_transaction(date: String,
                         date_format: &str,
                         payee: String,
                         category: Option<String>,
                         transaction_type: TransactionType,
                         amount: String,
                         status: TransactionStatus,
                         memo: Option<String>) -> Transaction {
        Transaction {
            date: Utc.datetime_from_str(&date, date_format).unwrap(),
            payee: payee::PayeeType::RawName(payee),
            category,
            transaction_type,
            amount,
            status,
            memo
        }
    }

    pub fn clean_payee(self, cleaned_name: String) -> Transaction {
        Transaction {
            payee: payee::PayeeType::ResolvedName(cleaned_name),
            ..self
        }
    }

    pub fn update_category(self, category: String) -> Transaction {
        Transaction {
            category: Option::Some(category),
            ..self
        }
    }
}