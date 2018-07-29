use chrono::prelude::*;

pub mod payee;
pub mod formats;
pub mod transaction_io;

#[derive(Debug)]
pub struct Transaction {
    date: DateTime<Utc>,
    payee: String,
    payee_name_type: payee::PayeeNameType,
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
            payee: InputCleaner::clean(payee),
            payee_name_type: payee::PayeeNameType::Raw,
            category: InputCleaner::clean(category),
            transaction_type,
            amount: InputCleaner::clean(amount),
            status,
            memo: InputCleaner::clean(memo),
        }
    }

    fn clean_payee(self, cleaned_name: String) -> Transaction {
        Transaction {
            payee: cleaned_name,
            payee_name_type: payee::PayeeNameType::Resolved,
            ..self
        }
    }

    fn update_category(self, category: String) -> Transaction {
        Transaction {
            category: Option::Some(category),
            ..self
        }
    }

    pub fn date(&self) -> &DateTime<Utc> {
        &self.date
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

#[cfg(test)]
mod tests {
    use super::{Transaction, TransactionType, TransactionStatus};
    use super::formats::input::ally_bank::{AllyTransaction, AllyTransactionType};
    use super::formats::input::citi::{CitiTransaction, CitiTransactionStatus};
    use super::payee::{PayeeNameType};
    use chrono::prelude::*;
    use ::parser;

    #[test]
    fn test_ally_transaction() {
        let ally = AllyTransaction::build(
            String::from("2018-01-01"),
            String::from("01:02:34"),
            String::from("-1234"),
            AllyTransactionType::Withdrawal,
            String::from("Description"));

        let transaction = Transaction::from(ally);
        assert_eq!(String::from("1234"), transaction.amount);
        assert_eq!(Utc.datetime_from_str("2018-01-01 01:02:34", "%Y-%m-%d %T").unwrap().timestamp(), transaction.date.timestamp());
        assert_eq!(TransactionType::Debit, transaction.transaction_type);
        assert_eq!(TransactionStatus::Cleared, transaction.status);

        let transaction = transaction.clean_payee(String::from("Payee A"));
        assert_eq!(PayeeNameType::Resolved, transaction.payee_name_type);
        assert_eq!(String::from("Payee A"), transaction.payee);

        let transaction = transaction.update_category(String::from("Category A"));
        assert_eq!(Option::Some(String::from("Category A")), transaction.category);
    }

    #[test]
    fn test_citi_transaction() {
        let citi = CitiTransaction::build(
            CitiTransactionStatus::Pending,
            String::from("06/01/2018"),
            String::from("Description"),
            Option::Some(String::from("1234")),
            Option::None);

        let transaction = Transaction::from(citi);
        assert_eq!(String::from("1234"), transaction.amount);
        assert_eq!(Utc.datetime_from_str("06/01/2018 00:00:00", "%m/%d/%Y %T").unwrap().timestamp(), transaction.date.timestamp());
        assert_eq!(TransactionType::Debit, transaction.transaction_type);
        assert_eq!(TransactionStatus::Pending, transaction.status);

        let transaction = transaction.clean_payee(String::from("Payee A"));
        assert_eq!(PayeeNameType::Resolved, transaction.payee_name_type);
        assert_eq!(String::from("Payee A"), transaction.payee);

        let transaction = transaction.update_category(String::from("Category A"));
        assert_eq!(Option::Some(String::from("Category A")), transaction.category);
    }

    #[test]
    fn test_parse_csv() {
        let ally_data = "Date, Time, Amount, Type, Description
2010-01-02,01:02:34,-1234,Withdrawal,Transfer to savings account";

        let mut ally_transactions: Vec<AllyTransaction> = parser::parse_csv_from_string(ally_data);

        let ally_transaction: AllyTransaction = ally_transactions.remove(0);
        let transaction = Transaction::from(ally_transaction);
        assert_eq!(String::from("1234"), transaction.amount);
        assert_eq!(Utc.datetime_from_str("2010-01-02 01:02:34", "%Y-%m-%d %T").unwrap().timestamp(), transaction.date.timestamp());
        assert_eq!(TransactionType::Debit, transaction.transaction_type);
        assert_eq!(TransactionStatus::Cleared, transaction.status);
    }
}