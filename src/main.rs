extern crate transaction_processor;

use transaction_processor::transaction::transaction_io::{TransactionIO};
use transaction_processor::config::{Config};

fn main() {
    let r = Config::new_and_parse_args().and_then(|c| {
        let transactions = TransactionIO::import(&c)?;
        TransactionIO::export(&c, transactions)
    });
    if let Err(e) = r {
        println!();
        println!("An error occurred, please try again.");
        println!("Error: {}", e);
    }
}
