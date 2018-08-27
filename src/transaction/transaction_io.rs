use transaction::{Transaction, TransactionStatus};
use super::formats::input::*;
use super::formats::output::*;
use std::io;
use arguments::{Arguments, SortOrder};
use std::fs::File;
use std::collections::HashSet;
use std::ffi::OsString;
use std::collections::HashMap;
use util::*;
use fantoccini::Client;
use web_driver::config::WebDriverConfig;
use web_driver;
use futures::Future;
use tokio_core;
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

    pub fn download(&self,
                    core: &mut tokio_core::reactor::Core,
                    mut client: &mut Client,
                    config: &WebDriverConfig,
                    args: &Arguments) -> HashSet<OsString> {
        let starting_files = get_files_in_dir(config.get_download_path());

        let downloader = self.importers.get(&args.src).unwrap();
        let account = if let Option::Some(ref a) = args.src_account {
            Option::Some(a.to_owned().into())
        } else {
            Option::None
        };
        let account = account.unwrap();
        downloader.download(core, &client, &account);

        // Wait for the transactions file to download
        let new_files = web_driver::wait_for_new_files(&mut client,
                                                       config.get_download_path(),
                                                       &starting_files).wait();
        if let Result::Ok(files) = new_files {
            return files;
        } else {
            return HashSet::new();
        }
    }

    pub fn import_files(&self, args: &Arguments, config: Option<&Config>, files: HashSet<OsString>) -> Vec<Transaction> {
        let mut transactions = Vec::new();
        for path in files {
            let r = {
                let f = File::open(path).expect("File not found");
                Box::new(io::BufReader::new(f))
            };
            transactions.extend(self.import_from_reader(args, config, r));
        }
        transactions
    }

    pub fn import(&self, args: &Arguments, config: Option<&Config>) -> Vec<Transaction> {
        let r: Box<io::Read> = match &args.src_file {
            Option::Some(f) => {
                let f = File::open(f).expect("File not found");
                Box::new(io::BufReader::new(f))
            },
            Option::None => Box::new(io::stdin()),
        };
        self.import_from_reader(args, config, r)
    }

    // Import from the reader and do filtering, sorting, etc.
    fn import_from_reader(&self, args: &Arguments, config: Option<&Config>, r: Box<io::Read>) -> Vec<Transaction> {
        let importer = self.importers.get(&args.src).unwrap();
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
    let account_id = args.src_account.as_ref().and_then(|x| {
        Option::Some(x.name.to_owned())
    });
    transactions.iter_mut().for_each(|t| {
        t.normalize_payee(account_id.to_owned(), config);
        t.categorize(args, account_id.to_owned(), config);
    });
    transactions
}