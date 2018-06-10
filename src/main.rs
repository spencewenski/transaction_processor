extern crate transaction_processor;
extern crate argparse;

use transaction_processor::arguments;
use transaction_processor::transaction::transaction_io::TransactionIO;

fn main() {
    let transaction_io = TransactionIO::new();

    let args = arguments::parse_args(transaction_io.list_importers(),
                                     transaction_io.list_exporters());

    let transactions = transaction_io.import(&args);

    transaction_io.export(&args, transactions);
}