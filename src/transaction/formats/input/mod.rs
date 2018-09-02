pub mod ally_bank;
pub mod citi;

use ::transaction::{Transaction};
use std::io;

pub trait TransactionImporter {
    fn import(&self, r: Box<io::Read>) -> Vec<Transaction>;
}