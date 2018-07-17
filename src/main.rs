extern crate transaction_processor;

use transaction_processor::web_driver::config::{WebDriverConfig, AutoDownload, MimeType};
use transaction_processor::web_driver::*;
use transaction_processor::transaction::transaction_io::{TransactionIO};
use transaction_processor::arguments::*;

fn main() {
    let transaction_io = TransactionIO::new();
    let args = parse_args(transaction_io.list_importers(),
                                     transaction_io.list_exporters());

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
            transaction_io.import_files(&args, files)
        } else {
            transaction_io.import(&args)
        }
    };
    transaction_io.export(&args, transactions);
}