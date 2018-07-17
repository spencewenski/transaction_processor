pub mod google_sheets;

use transaction::Transaction;
use std::io;

pub trait TransactionExporter {
    fn export(&self, w: Box<io::Write>, transactions: Vec<Transaction>);
}