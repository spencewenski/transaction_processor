use ::transaction::{Transaction, TransactionStatus, TransactionType};

// Column titles in Ally exports are prefixed with a space
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
    pub fn build_transaction(date: String,
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

impl From<AllyTransaction> for Transaction {
    fn from(ally: AllyTransaction) -> Self {
        let date_string = ally.date + " " + &ally.time;
        Transaction::build_transaction(date_string,
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