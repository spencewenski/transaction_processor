pub mod google_sheets;

use super::super::Transaction;
use std::io;

pub trait TransactionExporter {
    fn new() -> Box<Fn(Box<io::Write>, Vec<Transaction>)> {
        Box::new(|w: Box<io::Write>, transactions: Vec<Transaction>| {
            Self::export(w, transactions)
        })
    }
    fn export(w: Box<io::Write>, transactions: Vec<Transaction>);
}