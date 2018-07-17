pub mod ally_bank;
pub mod citi;

use ::transaction::{Transaction};
use std::io;
use fantoccini::Client;
use tokio_core;

pub trait TransactionImporter {
    fn import(&self, r: Box<io::Read>) -> Vec<Transaction>;
    fn download(&self,
                core: &mut tokio_core::reactor::Core,
                client: &Client,
                account: &Option<String>);
}