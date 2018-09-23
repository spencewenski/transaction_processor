use argparse::{ArgumentParser, Store, StoreOption, StoreConst};
use std::str::FromStr;
use super::{Sort, SortOrder, SortBy};

#[derive(Debug, Default)]
pub struct Arguments {
    pub config_file: String,
    pub src_account: String,
    pub dst_format: String,
    pub src_file: Option<String>,
    pub dst_file: Option<String>,
    pub sort: Option<Sort>,
    pub include_header: Option<bool>,
    pub ignore_pending: Option<bool>,
    pub skip_prompts: Option<bool>,
}

impl Arguments {
    pub fn parse_args() -> Arguments {
        let mut args: Arguments = Default::default();
        let mut sort_by: Option<String> = Default::default();
        let mut sort_order: Option<String> = Default::default();
        {
            let mut ap = ArgumentParser::new();
            ap.set_description("Transaction processor. \
            Command line arguments override any values that are also present in the config file.");

            ap.refer(&mut args.config_file)
                .add_option(&["--config-file"],
                            Store,
                            "Name of the config file.")
                .required();

            ap.refer(&mut args.src_account)
                .add_option(&["-a", "--src-account"],
                            Store,
                            "Id of the account")
                .required();

            ap.refer(&mut args.dst_format)
                .add_option(&["-d", "--dst-format"],
                            Store,
                            "Id of the destination data format")
                .required();

            ap.refer(&mut args.src_file)
                .add_option(&["-i", "--src-file"],
                            StoreOption,
                            "Source file");

            ap.refer(&mut args.dst_file)
                .add_option(&["-o", "--dst-file"],
                            StoreOption,
                            "Destination file");

            ap.refer(&mut sort_by)
                .add_option(&["--sort-by"],
                            StoreOption,
                            "What to sort the output by.");

            ap.refer(&mut sort_order)
                .add_option(&["--sort-order"],
                            StoreOption,
                            "Order in which to sort the output");

            ap.refer(&mut args.include_header)
                .add_option(&["--include-header"],
                            StoreConst(Option::Some(true)),
                            "Include header in the csv output.")
                .add_option(&["--exclude-header"],
                            StoreConst(Option::Some(false)),
                            "Exclude header in the csv output.");

            ap.refer(&mut args.ignore_pending)
                .add_option(&["--ignore-pending"],
                            StoreConst(Option::Some(true)),
                            "Ignore pending transactions. Defaults to false");

            ap.refer(&mut args.skip_prompts)
                .add_option(&["--skip-prompts"],
                            StoreConst(Option::Some(true)),
                            "Skip any prompts for optional user input.");

            ap.parse_args_or_exit();
        }
        args.set_sort(sort_by, sort_order);
        args
    }

    fn set_sort(&mut self, sort_by: Option<String>, sort_order: Option<String>) {
        let mut builder = SortBuilder::new();
        sort_by.and_then(|sort_by| {
            if let Ok(sort_by) = SortBy::from_str(&sort_by) {
                builder.sort_by(sort_by);
                sort_order
            } else {
                Option::None
            }
        }).and_then(|sort_order| {
            if let Ok(sort_order) = SortOrder::from_str(&sort_order) {
                builder.order(sort_order);
            }
            Option::Some(())
        }).and_then(|_| {
            self.sort = Option::Some(builder.build());
            Option::Some(())
        });
    }
}

struct SortBuilder {
    sort_by: SortBy,
    order: SortOrder,
}

impl SortBuilder {
    fn new() -> SortBuilder {
        SortBuilder {
            sort_by: SortBy::Date,
            order: SortOrder::Ascending,
        }
    }

    fn build(self) -> Sort {
        Sort {
            sort_by: self.sort_by,
            order: self.order,
        }
    }

    fn sort_by(&mut self, sort_by: SortBy) -> &mut SortBuilder {
        self.sort_by = sort_by;
        self
    }

    fn order(&mut self, order: SortOrder) -> &mut SortBuilder {
        self.order = order;
        self
    }
}

impl FromStr for SortBy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "date" => Ok(SortBy::Date),
            _ => Err(())
        }
    }
}

impl FromStr for SortOrder {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "ascending" => Ok(SortOrder::Ascending),
            "descending" => Ok(SortOrder::Descending),
            _ => Err(())
        }
    }
}
