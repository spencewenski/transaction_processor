use transaction::{Transaction, TransactionStatus, TransactionType};
use super::TransactionImporter;
use std::io;
use parser;
use tokio_core;
use fantoccini::{Locator, Element, Client, Form};
use url::Url;
use regex::Regex;
use futures::Future;

// Column titles input Ally exports are prefixed with a space
#[derive(Debug, Deserialize)]
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
impl AllyTransactionImporter {
    pub fn new() -> AllyTransactionImporter { AllyTransactionImporter{} }
}

impl TransactionImporter for AllyTransactionImporter {
    fn import(&self, r: Box<io::Read>) -> Vec<Transaction> {
        let transactions : Vec<AllyTransaction> = parser::parse_csv_from_reader(r);
        transactions.into_iter().map(|t| {
            Transaction::from(t)
        }).collect()
    }

    fn download(&self,
                core: &mut tokio_core::reactor::Core,
                client: &Client,
                account: &Option<String>) {
        let client = client;

        // now let's set up the sequence of steps we want the browser to take
        // first, go to the Wikipedia page for Foobar
        // login
        let f = client.goto("https://www.ally.com/")
            .and_then( |c| {
                println!("Finding account type element");
                c.find(Locator::Id("account"))
            })
            .and_then(|x| {
                println!("Selecting account type");
                x.select_by_value("aob")
            });

        core.run(f).unwrap();

        println!("Finding login form");
        let f = client.form(Locator::Css("form[data-id=\"storefront-login\"]"))
            .and_then(|f| {
                println!("Filling in username");
                assert!(false, "Need to set username and password");
                f.set_by_name("username", "username")
            })
            .and_then(|f| {
                println!("Filling in password");
                f.set_by_name("password", "password")
            })
            .and_then(|f| {
                println!("Submitting form");
                f.submit()
            });

        core.run(f).unwrap();

        // Open download button for first account
        let f = client.wait_for_find(Locator::Id("accounts-menu-item"))
            .and_then(|e| {
                println!("Clicking accounts menu");
                e.click()
            });

        core.run(f).unwrap();

        println!("Finding account list items (?)");
        let f =  client.wait_for_find(Locator::Css(".account-list"))
            .and_then(|e : Element | {
                println!("Getting raw html for account list");
                (client.current_url(), e.html(true))
            })
            .and_then(|url_html : (Url, String) | {
                let html = url_html.1.to_lowercase();
                // get accounts
                let re = Regex::new(r"(<a.*href.*#/bank/accounts/details/[0-9]+)").unwrap();
                let mut accounts_raw : Vec<String> = Vec::new();
                for cap in re.captures_iter(&html) {
                    let s = cap.get(1).unwrap().as_str();
                    accounts_raw.push(s.to_string());
                }
                // remove first part of account link
                let mut accounts : Vec<String> = Vec::new();
                let re = Regex::new(r"(#/bank/accounts/details/[0-9]+)").unwrap();
                for account in accounts_raw {
                    for cap in re.captures_iter(&account) {
                        let s = cap.get(1).unwrap().as_str();
                        accounts.push(s.to_string());
                    }
                }
                if !accounts.is_empty() {
                    client.goto(&url_html.0.join(accounts.get(0).unwrap()).unwrap().to_string())
                } else {
                    client.goto("https://www.ally.com/")
                }
            })
            .and_then(|_| {
                client.wait_for_find(Locator::Id("accounts-menu-item"))
            })
            .and_then(|e| {
                println!("Closing accounts menu");
                e.click()
            })
            .and_then(|_| {
                println!("Finding download button");
                client.wait_for_find(Locator::Css("a[aria-label=\"Download\"]"))
            })
            .and_then(|e| {
                println!("Clicking download button");
                e.click()
            });
        core.run(f).unwrap();

        // Download CSV for account

        let f = client.wait_for_find(Locator::Id("select-file-format"))
            .and_then(|e| {
                println!("Selecting CSV file format");
                e.select_by_value(".csv")
            })
            .and_then(|_| {
                client.wait_for_find(Locator::Id("select-date-range"))
            })
            .and_then(|e| {
                println!("Selecting date range");
                e.select_by_value("30")
            })
            .and_then(|_| {
                println!("Finding form");
                client.form(Locator::Css("form"))
            })
            .and_then(|f: Form | {
                println!("Submitting form");
                f.submit()
            });
        core.run(f).unwrap();
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