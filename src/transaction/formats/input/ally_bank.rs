use ::transaction::{Transaction, TransactionStatus, TransactionType};
use super::TransactionImporter;
use std::io;
use ::parser;

// Column titles input Ally exports are prefixed with a space
#[derive(Debug, Serialize, Deserialize)]
pub struct AllyTransaction {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = " Time")]
    time: String,
    #[serde(rename = " Amount")]
    amount: String,
    #[serde(rename = " Type")]
    transaction_type: AllyTransactionType,
    #[serde(rename = " Description")]
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AllyTransactionType {
    Withdrawal,
    Deposit,
}

impl AllyTransaction {
    pub fn build(date: String,
                 time: String,
                 amount: String,
                 transaction_type: AllyTransactionType,
                 description: String) -> AllyTransaction {
        AllyTransaction {
            date,
            time,
            amount,
            transaction_type,
            description
        }
    }
}

pub struct AllyTransactionImporter;
impl TransactionImporter for AllyTransactionImporter {
    fn import(r: Box<io::Read>) -> Vec<Transaction> {
        let transactions : Vec<AllyTransaction> = parser::parse_csv_from_reader(r);
        transactions.into_iter().map(|t| {
            Transaction::from(t)
        }).collect()
    }
}

impl From<AllyTransaction> for Transaction {
    fn from(ally: AllyTransaction) -> Self {
        let date_string = ally.date + " " + &ally.time;
        Transaction::build(date_string,
                           "%Y-%m-%d %T",
                           ally.description,
                           Option::None,
                           TransactionType::from(ally.transaction_type),
                           ally.amount.trim().trim_left_matches('-').to_string(),
                           TransactionStatus::Cleared,
                           Option::None)
    }
}

impl From<AllyTransactionType> for TransactionType {
    fn from(transaction_type: AllyTransactionType) -> Self {
        match transaction_type {
            AllyTransactionType::Withdrawal => TransactionType::Debit,
            AllyTransactionType::Deposit => TransactionType::Credit,
        }
    }
}