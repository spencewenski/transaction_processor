extern crate transaction_processor;

use transaction_processor::web_driver::config::{WebDriverConfig, AutoDownload, MimeType};
use transaction_processor::web_driver::*;
use transaction_processor::transaction::transaction_io::{TransactionIO};
use transaction_processor::arguments::*;
use transaction_processor::util;
use transaction_processor::config::Config;

fn main() {
    let transaction_io = TransactionIO::new();
    let args = parse_args(transaction_io.list_importers(),
                                     transaction_io.list_exporters());

    let c = args.config_file.as_ref().and_then(|x| {
        Option::Some(util::reader_from_file_name(&x))
    }).and_then(|x | {
        match Config::from_reader(x) {
            Ok(c) => Option::Some(c),
            Err(e) => {
                println!("Unable to read config file; {}", e);
                Option::None
            }
        }
    });

    let transactions = {
        if let SourceType::Website = args.src_type {
            let config = WebDriverConfig::build(AutoDownload::True(MimeType::TextCsv),
                                                false, "http://localhost:4444");
            let mut core = create_core();
            let mut client = create_client(&mut core, &config);
            let files = transaction_io.download(&mut core, &mut client, &config, &args);
            if let Some(fin) = client.close() {
                // and wait for cleanup to finish
                core.run(fin).unwrap();
            }
            transaction_io.import_files(&args, c.as_ref(), files)
        } else {
            transaction_io.import(&args, c.as_ref())
        }
    };

    transaction_io.export(&args, transactions);
}