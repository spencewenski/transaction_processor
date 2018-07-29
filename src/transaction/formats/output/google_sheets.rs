use transaction::{Transaction, TransactionStatus, TransactionType};
use super::TransactionExporter;
use std::io;
use parser;

#[derive(Debug, Serialize)]
pub struct GoogleSheetsTransaction {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Payee")]
    payee: String,
    #[serde(rename = "Category")]
    category: Option<String>,
    #[serde(rename = "Debit")]
    debit: Option<String>,
    #[serde(rename = "Credit")]
    credit: Option<String>,
    #[serde(rename = "Status")]
    status: GoogleSheetsTransactionStatus,
    #[serde(rename = "Memo")]
    memo: Option<String>,
}

#[derive(Debug, Serialize)]
enum GoogleSheetsTransactionStatus {
    Pending,
    Cleared,
}

impl GoogleSheetsTransaction {
    fn build(date: String,
             payee: String,
             category: Option<String>,
             debit: Option<String>,
             credit: Option<String>,
             status: GoogleSheetsTransactionStatus,
             memo: Option<String>) -> GoogleSheetsTransaction {
        GoogleSheetsTransaction {
            date,
            payee,
            category,
            debit,
            credit,
            status,
            memo,
        }
    }
}

pub struct GoogleSheetsTransactionExporter;
impl GoogleSheetsTransactionExporter {
    pub fn new() -> GoogleSheetsTransactionExporter { GoogleSheetsTransactionExporter{} }
}

impl TransactionExporter for GoogleSheetsTransactionExporter {
    fn export(&self, w: Box<io::Write>, transactions: Vec<Transaction>, include_header: bool) {
        let transactions: Vec<GoogleSheetsTransaction> = transactions.into_iter().map(|t| {
            Transaction::into(t)
        }).collect();
        parser::write_csv_to_writer(transactions, include_header, w);
    }
}

impl From<Transaction> for GoogleSheetsTransaction {
    fn from(transaction: Transaction) -> Self {
        let mut debit: Option<String> = Option::None;
        let mut credit: Option<String> = Option::None;
        match transaction.transaction_type {
            TransactionType::Debit => debit = Option::Some(transaction.amount),
            TransactionType::Credit => credit = Option::Some(transaction.amount),
        }

        GoogleSheetsTransaction::build(
            format!("{}", transaction.date.format("%m/%d/%Y")),
            transaction.payee,
            transaction.category,
            debit,
            credit,
            TransactionStatus::into(transaction.status),
            transaction.memo,
        )
    }
}

impl From<TransactionStatus> for GoogleSheetsTransactionStatus {
    fn from(status: TransactionStatus) -> Self {
        match status {
            TransactionStatus::Pending => GoogleSheetsTransactionStatus::Pending,
            TransactionStatus::Cleared => GoogleSheetsTransactionStatus::Cleared,
        }
    }
}