extern crate transaction_processor;
extern crate argparse;

use transaction_processor::transaction::{Transaction};
use transaction_processor::transaction::parser;
use transaction_processor::transaction::formats::ally_bank::AllyTransaction;
use transaction_processor::transaction::formats::google_sheets::GoogleSheetsTransaction;
use transaction_processor::arguments;

fn main() {
    let args = arguments::parse_args();
    println!("{:?}", args);

    let transactions : Vec<AllyTransaction> = parser::parse_csv();

    let transactions : Vec<GoogleSheetsTransaction> = transactions.into_iter()
        .map(|t| {
            Transaction::from(t)
        })
        .map(|t| {
            Transaction::into(t)
        }).collect();

    parser::write_csv(transactions, true);
}