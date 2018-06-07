use chrono::prelude::*;

pub mod payee;
pub mod formats;

#[derive(Debug)]
struct Transaction {
    date: DateTime<Utc>,
    payee: payee::PayeeType,
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

    fn clean_payee(self, cleaned_name: String) -> Transaction {
        Transaction {
            payee: payee::PayeeType::ResolvedName(cleaned_name),
            ..self
        }
    }

    fn update_category(self, category: String) -> Transaction {
        Transaction {
            category: Option::Some(category),
            ..self
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{Transaction, TransactionType, TransactionStatus};
    use super::formats::ally_bank::{AllyTransaction, AllyTransactionType};
    use super::payee::{PayeeType};
    use chrono::prelude::*;

    #[test]
    fn it_works() {
        let ally = AllyTransaction::build_transaction(
            String::from("2018-06-01"),
            String::from("01:01:54"),
            String::from("-4874"),
            AllyTransactionType::Withdrawal,
            String::from("Internet transfer to Online Savings account XXXXXX5489"));

        let transaction = Transaction::from(ally);
        assert_eq!(transaction.amount, String::from("4874"));
        assert_eq!(transaction.date.timestamp(), Utc.datetime_from_str("2018-06-01 01:01:54", "%Y-%m-%d %T").unwrap().timestamp());
        assert_eq!(transaction.transaction_type, TransactionType::Debit);
        assert_eq!(transaction.status, TransactionStatus::Cleared);

        let transaction = transaction.clean_payee(String::from("Ally Savings"));
        assert_eq!(transaction.payee, PayeeType::ResolvedName(String::from("Ally Savings")));

        let transaction = transaction.update_category(String::from("Category A"));
        assert_eq!(transaction.category, Option::Some(String::from("Category A")));
    }
}