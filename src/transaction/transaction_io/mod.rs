use transaction::{Transaction, TransactionStatus};
use std::io;
use std::fs::File;
use config::{Config, SortOrder};

mod formats;

pub struct TransactionIO {
}

impl TransactionIO {
    pub fn import(config: &Config) -> Result<Vec<Transaction>, String> {
        let r: Box<io::Read> = match config.src_file() {
            Option::Some(f) => {
                let f = File::open(f).map_err(|e| {
                    format!("An error occurred while trying to open file [{}]: {}", f, e)
                })?;
                Box::new(io::BufReader::new(f))
            },
            Option::None => Box::new(io::stdin()),
        };
        let transactions = formats::import_from_configurable_format(r, config.src_format())?;
        let transactions = filter(config, transactions);
        let transactions = normalize_and_categorize(config, transactions);
        Ok(transactions)
    }

    pub fn export(config: &Config, transactions: Vec<Transaction>) -> Result<(), String> {
        // Sort transactions just before exporting
        let transactions = sort(config, transactions);
        let w: Box<io::Write> = match config.dst_file() {
            Option::Some(f) => {
                let f = File::create(f).map_err(|e| {
                    format!("An error occurred while trying to open file [{}]: {}", f, e)
                })?;
                Box::new(io::BufWriter::new(f))
            },
            Option::None => Box::new(io::stdout())
        };
        formats::export_to_configurable_format(w, config, config.dst_format(), transactions)?;
        Ok(())
    }
}

fn filter(config: &Config, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    // We only support filtering pending transactions now, so just bail if we don't want to
    // ignore pending.
    if !config.ignore_pending() {
        return transactions;
    }
    transactions.retain(|e| {
        e.status == TransactionStatus::Cleared
    });
    transactions
}

fn normalize_and_categorize(config: &Config, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    transactions.iter_mut().for_each(|t| {
        t.normalize_payee(config);
        t.categorize(config);
    });
    transactions
}

fn sort(config: &Config, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    if let Option::Some(ref sort) = config.sort() {
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