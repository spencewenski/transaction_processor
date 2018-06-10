use ::transaction::{Transaction, TransactionStatus, TransactionType};
use super::TransactionImporter;
use std::io;
use ::parser;

#[derive(Debug, Deserialize)]
pub struct CitiTransaction {
    #[serde(rename = "Status")]
    status: CitiTransactionStatus,
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Debit")]
    debit: Option<String>,
    #[serde(rename = "Credit")]
    credit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum CitiTransactionStatus {
    Pending,
    Cleared
}

impl CitiTransaction {
    pub fn build(status: CitiTransactionStatus,
                 date: String,
                 description: String,
                 debit: Option<String>,
                 credit: Option<String>) -> CitiTransaction {
        CitiTransaction {
            status,
            date,
            description,
            debit,
            credit
        }
    }
}

pub struct CitiTransactionImporter;
impl TransactionImporter for CitiTransactionImporter {
    fn import(r: Box<io::Read>) -> Vec<Transaction> {
        let transactions : Vec<CitiTransaction> = parser::parse_csv_from_reader(r);
        transactions.into_iter().map(|t| {
            Transaction::from(t)
        }).collect()
    }
}

impl From<CitiTransaction> for Transaction {
    fn from(citi: CitiTransaction) -> Self {
        let type_and_amount = get_transaction_type_and_amount(citi.debit, citi.credit);
        Transaction::build(citi.date + " 00:00:00",
                           "%m/%d/%Y %T",
                           citi.description,
                           Option::None,
                           type_and_amount.0,
                           type_and_amount.1.unwrap(),
                           TransactionStatus::from(citi.status),
                           Option::None)
    }
}

impl From<CitiTransactionStatus> for TransactionStatus {
    fn from(status: CitiTransactionStatus) -> TransactionStatus {
        match status {
            CitiTransactionStatus::Pending => TransactionStatus::Pending,
            CitiTransactionStatus::Cleared => TransactionStatus::Cleared,
        }
    }
}

fn get_transaction_type_and_amount(debit: Option<String>, credit: Option<String>) -> (TransactionType, Option<String>) {
    match debit {
        Option::Some(d) => (TransactionType::Debit, Option::Some(d)),
        _ => {
            match credit {
                Option::Some(c) => (TransactionType::Credit, Option::Some(c)),
                _ => (TransactionType::Credit, Option::Some(String::from("0")))
            }
        }
    }
}
