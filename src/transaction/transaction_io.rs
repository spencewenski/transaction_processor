use transaction::Transaction;
use super::formats::input::{TransactionImporter};
use super::formats::input::ally_bank::AllyTransactionImporter;
use super::formats::input::citi::CitiTransactionImporter;
use super::formats::output::{TransactionExporter};
use super::formats::output::google_sheets::GoogleSheetsTransactionExporter;
use std::io;
use std::collections::BTreeMap;
use arguments::Arguments;
use std::fs::File;

pub struct TransactionIO {
    importers: BTreeMap<String, Box<Fn(Box<io::Read>) -> Vec<Transaction>>>,
    exporters: BTreeMap<String, Box<Fn(Box<io::Write>, Vec<Transaction>)>>,
}

impl TransactionIO {
    pub fn new() -> TransactionIO {
        let mut importers: BTreeMap<String, Box<Fn(Box<io::Read>) -> Vec<Transaction>>> = BTreeMap::new();
        importers.insert(String::from("ally"), AllyTransactionImporter::new());
        importers.insert(String::from("citi"), CitiTransactionImporter::new());

        let mut exporters: BTreeMap<String, Box<Fn(Box<io::Write>, Vec<Transaction>)>> = BTreeMap::new();
        exporters.insert(String::from("google"), GoogleSheetsTransactionExporter::new());

        TransactionIO {
            importers,
            exporters,
        }
    }

    pub fn list_importers(&self) -> Vec<&String> {
        let mut importers_list: Vec<&String> = Vec::new();
        self.importers.iter().for_each(|x| {
            importers_list.push(x.0);
        });
        importers_list
    }

   pub fn list_exporters(&self) -> Vec<&String> {
        let mut exporters_list: Vec<&String> = Vec::new();
        self.exporters.iter().for_each(|x| {
            exporters_list.push(x.0);
        });
        exporters_list
    }

    pub fn import(&self, args: &Arguments) -> Vec<Transaction> {
        let importer = self.importers.get(&args.src_format).unwrap();
        let r: Box<io::Read> = match &args.src_file {
            Option::Some(f) => {
                let f = File::open(f).expect("File not found");
                Box::new(io::BufReader::new(f))
            },
            Option::None => Box::new(io::stdin()),
        };
        importer(r)
    }

    pub fn export(&self, args: &Arguments, transactions: Vec<Transaction>) {
        let exporter = self.exporters.get(&args.dst_format).unwrap();
        let w: Box<io::Write> = match &args.dst_file {
            Option::Some(f) => {
                let f = File::create(f).expect("Unable to open file");
                Box::new(io::BufWriter::new(f))
            },
            Option::None => Box::new(io::stdout())
        };
        exporter(w, transactions);
    }
}