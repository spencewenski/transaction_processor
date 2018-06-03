extern crate chrono;
extern crate csv;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use chrono::prelude::*;
use std::collections::HashMap;
use std::io;

// DateTime Serializer
mod only_date_format {
    use chrono::{DateTime, Utc};
    use serde::{Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let s = format!("{}", date.format("%Y-%m-%d"));
        serializer.serialize_str(&s)
    }
}

// Transaction mod
struct Payees {
    // payee name -> Payee
    payees: HashMap<String, Payee>,
    // raw payee name -> clean payee name
    raw_payees: HashMap<String, String>,
}

impl Payees {
    fn new() -> Payees {
        Payees {
            payees: HashMap::new(),
            raw_payees: HashMap::new(),
        }
    }

    fn get_payee(&self, payee_name: &str) -> Option<&Payee> {
        self.payees.get(payee_name)
    }

    fn get_payee_name(&self, raw_payee_name: &str) -> Option<&String> {
        self.raw_payees.get(raw_payee_name)
    }
}

#[derive(Debug, Clone)]
struct Payee {
    name: String,
    default_category: Option<Category>,
}

impl Payee {
    fn new(name: String, default_category: Option<Category>) -> Payee {
        Payee {
            name,
            default_category,
        }
    }
}

#[derive(Debug, Clone)]
struct Category {
    name: String,
}

impl Category {
    fn new(name: String) -> Category {
        Category {
            name,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
enum TransactionStatus {
    Pending,
    Cleared,
}

#[derive(Debug, Serialize, Clone)]
struct Transaction {
    #[serde(rename = "Date")]
    #[serde(with = "only_date_format")]
    date: DateTime<Utc>,
    #[serde(skip)]
    raw_payee_name: String,
    #[serde(rename = "Payee")]
    payee: String,
    #[serde(rename = "Category")]
    category: Option<String>,
    #[serde(rename = "Debit")]
    debit: Option<String>,
    #[serde(rename = "Debit")]
    credit: Option<String>,
    #[serde(rename = "Status")]
    status: TransactionStatus,
    #[serde(rename = "Memo")]
    memo: Option<String>,
}

// Print all transactions as CSV
fn print_transaction_csv(transactions: &Vec<Transaction>, has_headers: bool) {
    let mut writer = csv::WriterBuilder::new()
        .has_headers(has_headers)
        .from_writer(io::stdout());
    for transaction in transactions {
        writer.serialize(transaction).unwrap();
    }
}

// Populate the payee info for a transaction
fn populate_payee_info(transaction: &Transaction, payees: &Payees) -> Transaction {
    let mut t = transaction.clone();
    t.payee = payees.get_payee_name(&transaction.raw_payee_name)
        .map_or(transaction.raw_payee_name.to_owned(),|payee_name| {
            payee_name.to_owned()
        });
    t.category = payees.get_payee(&transaction.payee)
        .map(|payee| {
            payee.clone().default_category.unwrap_or(Category::new(String::from("")))
        })
        .map(|category| {
            category.name
        });
    t
}

// Populate the payee info for a list of transactions
fn populate_all_payee_info(transactions: Vec<Transaction>, payees: &Payees) -> Vec<Transaction> {
    transactions.iter().map(|transaction| {
        populate_payee_info(transaction, payees)
    }).collect()
}

// Ally transactions mod
#[derive(Debug, Deserialize)]
#[serde(rename = "Type")]
enum AllyTransactionType {
    Withdrawal,
    Deposit,
}

#[derive(Debug, Deserialize)]
struct AllyTransaction {
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

impl<'a> From<&'a AllyTransaction> for Transaction {
    fn from(ally: &'a AllyTransaction) -> Self {
        let amount = ally.amount.trim().trim_left_matches('-').to_string();
        let debit = match ally.transaction_type {
            AllyTransactionType::Withdrawal => Option::Some(amount.to_string()),
            _ => Option::None,
        };
        let credit = match ally.transaction_type {
            AllyTransactionType::Deposit => Option::Some(amount),
            _ => Option::None,
        };
        Transaction {
            date: Utc.datetime_from_str(&(ally.date.to_owned() + " " + &ally.time), "%Y-%m-%d %T").unwrap(),
            raw_payee_name: ally.description.to_owned(),
            payee: String::from(""),
            category: Option::None,
            debit,
            credit,
            status: TransactionStatus::Cleared,
            memo: Option::None,
        }
    }
}

trait CsvParser {
    fn parse() -> Vec<Transaction> {
        Self::parse_from_reader(io::stdin())
    }
    fn parse_from_reader<R>(r: R) -> Vec<Transaction> where R : io::Read;
}

impl CsvParser for AllyTransaction {
    fn parse_from_reader<R>(r: R) -> Vec<Transaction> where R : io::Read {
        let mut reader = csv::Reader::from_reader(r);
        let mut transactions = Vec::new();
        for result in reader.deserialize() {
            let ally_transaction: AllyTransaction = result.unwrap();
            println!("{:?}", ally_transaction);
            transactions.push(ally_transaction);
        }
        transactions.iter().map(|ally_transaction| {
            Transaction::from(ally_transaction)
        }).collect()
    }
}

fn main() {
    let mut payees = Payees::new();
    let payee = Payee::new(
        String::from("Citi Credit Card"),
        Option::Some(Category::new(String::from("Citi Credit Card"))));
    payees.raw_payees.insert(String::from("CITI CARD ONLINE PAYMENT"), payee.name.to_owned());
    payees.payees.insert(payee.name.to_owned(), payee);

    let mut transactions = AllyTransaction::parse();
    transactions = populate_all_payee_info(transactions, &payees);
    transactions.sort_by_key(|x| {
        x.date
    });

    println!();

    print_transaction_csv(&transactions, true);
    println!();
    print_transaction_csv(&transactions, false);
}
