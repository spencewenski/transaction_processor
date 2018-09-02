use transaction::{Transaction, TransactionStatus};
use super::formats::input::*;
use super::formats::output::*;
use std::io;
use arguments::{Arguments, SortOrder};
use std::fs::File;
use std::collections::HashMap;
use config::Config;

pub struct TransactionIO {
    importers: HashMap<String, Box<TransactionImporter>>,
    exporters: HashMap<String, Box<TransactionExporter>>,
}

impl TransactionIO {
    pub fn new() -> TransactionIO {
        let mut importers: HashMap<String, Box<TransactionImporter>> = HashMap::new();
        importers.insert(String::from("ally"), Box::new(ally_bank::AllyTransactionImporter::new()));
        importers.insert(String::from("citi"), Box::new(citi::CitiTransactionImporter::new()));

        let mut exporters: HashMap<String, Box<TransactionExporter>> = HashMap::new();
        exporters.insert(String::from("google"), Box::new(google_sheets::GoogleSheetsTransactionExporter::new()));

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

    pub fn import(&self, args: &Arguments, config: Option<&Config>) -> Vec<Transaction> {
        let r: Box<io::Read> = match &args.src_file {
            Option::Some(f) => {
                let f = File::open(f).expect("File not found");
                Box::new(io::BufReader::new(f))
            },
            Option::None => Box::new(io::stdin()),
        };
        let importer = self.importers.get(&args.src_format).unwrap();
        let transactions = importer.import(r);
        let transactions = filter(args, transactions);
        let transactions = sort(args, transactions);
        normalize_and_categorize(args, config, transactions)
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
        exporter.export(w, transactions, args.include_header);
    }
}

fn filter(args: &Arguments, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    // We only support filtering pending transactions now, so just bail if we don't want to
    // ignore pending.
    if !args.ignore_pending {
        return transactions;
    }
    transactions.retain(|e| {
        e.status == TransactionStatus::Cleared
    });
    transactions
}

fn sort(args: &Arguments, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    if let Option::Some(ref sort) = &args.sort {
        transactions.sort_by(|a, b| {
            if SortOrder::Ascending == sort.order {
                a.date().cmp(&b.date())
            } else {
                a.date().cmp(&b.date()).reverse()
            }
        });
    }
    transactions
}

fn normalize_and_categorize(args: &Arguments, config: Option<&Config>, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    if let Option::Some(ref a) = args.src_account {
        transactions.iter_mut().for_each(|t| {
            t.normalize_payee(a, config);
            t.categorize(args, a, config);
        });
    }
    transactions
}