extern crate transaction_processor;

use transaction_processor::transaction::transaction_io::{TransactionIO};
use transaction_processor::arguments::*;
use transaction_processor::util;
use transaction_processor::config::Config;

fn main() {
    let args = parse_args();

    args.config_file.as_ref().and_then(|x| {
        Option::Some(util::reader_from_file_name(&x))
    }).and_then(|x | {
        match Config::from_reader(x) {
            Ok(c) => Option::Some(c),
            Err(e) => {
                println!("Unable to read config file; {}", e);
                Option::None
            }
        }
    }).and_then(|c| {
        let transactions = TransactionIO::import(&args, &c);

        TransactionIO::export(&args, &c, transactions);

        Option::Some(())
    });
}
