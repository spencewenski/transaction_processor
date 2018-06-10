extern crate transaction_processor;
extern crate argparse;

use transaction_processor::transaction::{Transaction};
use transaction_processor::transaction::formats::input::ally_bank::{AllyTransactionImporter};
use transaction_processor::transaction::formats::input::citi::{CitiTransactionImporter};
use transaction_processor::transaction::formats::input::{TransactionImporter};
use transaction_processor::transaction::formats::output::google_sheets::{GoogleSheetsTransactionExporter};
use transaction_processor::transaction::formats::output::{TransactionExporter};
use transaction_processor::arguments;
use std::collections::HashMap;
use std::fs::File;
use std::io;

fn main() {
    let args = arguments::parse_args();

    let mut importers: HashMap<String, Box<Fn(Box<io::Read>) -> Vec<Transaction>>> = HashMap::new();
    importers.insert(String::from("ally"), AllyTransactionImporter::new());
    importers.insert(String::from("citi"), CitiTransactionImporter::new());

    let mut exporters: HashMap<String, Box<Fn(Box<io::Write>, Vec<Transaction>)>> = HashMap::new();
    exporters.insert(String::from("google"), GoogleSheetsTransactionExporter::new());

    let importer = importers.get(&args.src_format).unwrap();
    let r: Box<io::Read> = match args.src_file {
        Option::Some(f) => {
            let f = File::open(f).expect("File not found");
            Box::new(io::BufReader::new(f))
        },
        Option::None => Box::new(io::stdin()),
    };
    let transactions = importer(r);

    let exporter = exporters.get(&args.dst_format).unwrap();
    let w: Box<io::Write> = match args.dst_file {
        Option::Some(f) => {
            let f = File::create(f).expect("Unable to open file");
            Box::new(io::BufWriter::new(f))
        },
        Option::None => Box::new(io::stdout())
    };
    exporter(w, transactions);
}