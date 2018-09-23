use transaction::{Transaction, TransactionStatus};
use std::io;
use arguments::{Arguments, SortOrder};
use std::fs::File;
use config::Config;

mod formats;

pub struct TransactionIO {
}

impl TransactionIO {
    pub fn import(args: &Arguments, config: &Config) -> Vec<Transaction> {
        let r: Box<io::Read> = match &args.src_file {
            Option::Some(f) => {
                let f = File::open(f).expect("File not found");
                Box::new(io::BufReader::new(f))
            },
            Option::None => Box::new(io::stdin()),
        };
        config.formats.get(&args.src_format).and_then(|f| {
            let transactions = formats::import_from_configurable_format(r, f);
            let transactions = filter(args, transactions);
            let transactions = sort(args, transactions);
            Option::Some(normalize_and_categorize(args, config, transactions))
        }).unwrap_or(Vec::default())
    }

    pub fn export(args: &Arguments, config: &Config, transactions: Vec<Transaction>) {
        let w: Box<io::Write> = match &args.dst_file {
            Option::Some(f) => {
                let f = File::create(f).expect("Unable to open file");
                Box::new(io::BufWriter::new(f))
            },
            Option::None => Box::new(io::stdout())
        };
        config.formats.get(&args.dst_format).and_then(|f| {
            formats::export_to_configurable_format(w, f, transactions);
            Option::Some(())
        });
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

fn normalize_and_categorize(args: &Arguments, config: &Config, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    if let Option::Some(ref a) = args.src_account {
        transactions.iter_mut().for_each(|t| {
            t.normalize_payee(a, config);
            t.categorize(args, a, config);
        });
    }
    transactions
}