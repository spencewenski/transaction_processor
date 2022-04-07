use anyhow::anyhow;
use config::{Config, SortBy, SortOrder};
use std::fs::File;
use std::io;
use transaction::{Transaction, TransactionStatus};

mod formats;

pub struct TransactionIO {}

impl TransactionIO {
    pub fn import(config: &Config) -> anyhow::Result<Vec<Transaction>> {
        let r: Box<dyn io::Read> = match config.src_file() {
            Option::Some(f) => {
                let f = File::open(f).map_err(|e| {
                    anyhow!(
                        "An error occurred while trying to open file [{}]: {}",
                        f.to_str().unwrap_or("Invalid file name"),
                        e
                    )
                })?;
                Box::new(io::BufReader::new(f))
            }
            Option::None => Box::new(io::stdin()),
        };
        let transactions = formats::import_from_configurable_format(r, config.src_format())?;
        let transactions = filter(config, transactions);
        let transactions = normalize_and_categorize(config, transactions);
        Ok(transactions)
    }

    pub fn export(config: &Config, transactions: Vec<Transaction>) -> anyhow::Result<()> {
        // Sort transactions just before exporting
        let transactions = sort(config, transactions);
        let w: Box<dyn io::Write> = match config.dst_file() {
            Option::Some(f) => {
                let f = File::create(f).map_err(|e| {
                    anyhow!(
                        "An error occurred while trying to open file [{}]: {}",
                        f.to_str().unwrap_or("Invalid file name"),
                        e
                    )
                })?;
                Box::new(io::BufWriter::new(f))
            }
            Option::None => Box::new(io::stdout()),
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
    transactions.retain(|e| e.status == TransactionStatus::Cleared);
    transactions
}

fn normalize_and_categorize(
    config: &Config,
    mut transactions: Vec<Transaction>,
) -> Vec<Transaction> {
    transactions.iter_mut().for_each(|t| {
        t.normalize_payee(config);
        t.categorize(config);
    });
    transactions
}

fn sort(config: &Config, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    if config.sort_by().is_none() || config.sort_order().is_none() {
        return transactions;
    }

    let get_data_fn = match config.sort_by().unwrap() {
        // todo: don't create an owned copy
        SortBy::Date => |t: &Transaction| t.date().to_owned(),
    };

    if let Option::Some(ref sort_order) = config.sort_order() {
        transactions.sort_by(|a, b| {
            if SortOrder::Ascending == *sort_order {
                get_data_fn(a).cmp(&get_data_fn(b))
            } else {
                get_data_fn(a).cmp(&get_data_fn(b)).reverse()
            }
        });
    }
    transactions
}
