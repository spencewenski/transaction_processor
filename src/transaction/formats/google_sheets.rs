use ::transaction::{Transaction, TransactionStatus, TransactionType};

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleSheetsTransaction {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Payee")]
    payee: String,
    #[serde(rename = "Category")]
    category: Option<String>,
    #[serde(rename = "Debit")]
    debit: Option<String>,
    #[serde(rename = "Debit")]
    credit: Option<String>,
    #[serde(rename = "Status")]
    status: GoogleSheetsTransactionStatus,
    #[serde(rename = "Memo")]
    memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum GoogleSheetsTransactionStatus {
    Pending,
    Cleared,
}

impl GoogleSheetsTransaction {
    fn build_transaction(date: String,
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

impl From<Transaction> for GoogleSheetsTransaction {
    fn from(transaction: Transaction) -> Self {
        let mut debit: Option<String> = Option::None;
        let mut credit: Option<String> = Option::None;
        match transaction.transaction_type {
            TransactionType::Debit => debit = Option::Some(transaction.amount),
            TransactionType::Credit => credit = Option::Some(transaction.amount),
        }

        GoogleSheetsTransaction::build_transaction(
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