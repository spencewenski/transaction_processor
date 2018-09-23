use argparse::{ArgumentParser, Store, StoreTrue, StoreFalse};
use std::str::FromStr;
use util;

#[derive(Debug)]
pub struct Arguments {
    pub src_account: Option<String>,
    pub dst_format: String,
    pub src_file: Option<String>,
    pub dst_file: Option<String>,
    pub sort: Option<Sort>,
    pub include_header: bool,
    pub ignore_pending: bool,
    pub skip_prompts: bool,
    pub config_file: Option<String>,
}

#[derive(Debug)]
pub enum SourceType {
    File,
    Website,
}

impl FromStr for SourceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            "file" => Ok(SourceType::File),
            "website" => Ok(SourceType::Website),
            _ => Err(())
        }
    }
}

impl From<Args> for Arguments {
    fn from(a: Args) -> Arguments {
        Arguments {
            src_account: util::get_optional_string(a.src_account),
            dst_format: a.dst_format,
            src_file: util::get_optional_string(a.src_file),
            dst_file: util::get_optional_string(a.dst_file),
            sort: get_sort(a.sort_by, a.sort_order),
            include_header: a.include_header,
            ignore_pending: a.ignore_pending,
            skip_prompts: a.skip_prompts,
            config_file: util::get_optional_string(a.config_file),
        }
    }
}

fn get_sort(sort_by: String, sort_order: String) -> Option<Sort> {
    if let Err(_) = SortBy::from_str(&sort_by) {
        return Option::None;
    }
    let mut builder = SortBuilder::new();
    if let Ok(x) = SortBy::from_str(&sort_by) {
        builder.sort_by(x);
    }
    if let Ok(x) = SortOrder::from_str(&sort_order) {
        builder.order(x);
    }
    Option::Some(builder.build())
}

#[derive(Debug)]
pub struct Sort {
    pub sort_by: SortBy,
    pub order: SortOrder,
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

#[derive(Debug, PartialEq)]
pub enum SortBy {
    Date,
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

#[derive(Debug, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
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

struct Args {
    src_account: String,
    dst_format: String,
    src_file: String,
    dst_file: String,
    sort_by: String,
    sort_order: String,
    include_header: bool,
    ignore_pending: bool,
    skip_prompts: bool,
    config_file: String,
}

impl Args {
    fn new() -> Args {
        Args {
            src_account: Default::default(),
            dst_format: Default::default(),
            src_file: Default::default(),
            dst_file: Default::default(),
            sort_by: Default::default(),
            sort_order: Default::default(),
            include_header: true,
            ignore_pending: false,
            skip_prompts: false,
            config_file: Default::default(),
        }
    }
}

pub fn parse_args() -> Arguments {
    let mut args = Args::new();
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
                        Store,
                        "Source file");

        ap.refer(&mut args.dst_file)
            .add_option(&["-o", "--dst-file"],
                        Store,
                        "Destination file");

        ap.refer(&mut args.sort_by)
            .add_option(&["--sort-by"],
                        Store,
                        "What to sort the output by.");

        ap.refer(&mut args.sort_order)
            .add_option(&["--sort-order"],
                        Store,
                        "Order in which to sort the output");

        ap.refer(&mut args.include_header)
            .add_option(&["--include-header"],
                        StoreTrue,
                        "Include header in the csv output.")
            .add_option(&["--exclude-header"],
                        StoreFalse,
                        "Exclude header in the csv output.");

        ap.refer(&mut args.ignore_pending)
            .add_option(&["--ignore-pending"],
                        StoreTrue,
                        "Ignore pending transactions. Defaults to false");

        ap.refer(&mut args.skip_prompts)
            .add_option(&["--skip-prompts"],
                        StoreTrue,
                        "Skip any prompts for optional user input.");

        ap.parse_args_or_exit();
    }
    Arguments::from(args)
}