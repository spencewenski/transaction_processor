pub mod ally_bank;
pub mod citi;

use super::super::Transaction;
use std::io;

pub trait TransactionImporter {
    fn new() -> Box<Fn(Box<io::Read>) -> Vec<Transaction>> {
        Box::new(|r: Box<io::Read>| {
            Self::import(r)
        })
    }
    fn import(r: Box<io::Read>) -> Vec<Transaction>;
}