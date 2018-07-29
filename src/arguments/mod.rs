use argparse::{ArgumentParser, Store, StoreTrue, StoreFalse};
use itertools::Itertools;
use std::str::FromStr;

#[derive(Debug)]
pub struct Arguments {
    pub src: String,
    pub src_type: SourceType,
    pub src_account: Option<String>,
    pub dst_format: String,
    pub src_file: Option<String>,
    pub dst_file: Option<String>,
    pub sort: Option<Sort>,
    pub include_header: bool,
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
            src: a.src,
            src_type: a.src_type,
            src_account: if a.src_account.len() != 0 {
                Option::Some(a.src_account)
            } else {
                Option::None
            },
            dst_format: a.dst_format,
            src_file: if a.src_file.len() != 0 {
                Option::Some(a.src_file)
            } else {
                Option::None
            },
            dst_file: if a.dst_file.len() != 0 {
                Option::Some(a.dst_file)
            } else {
                Option::None
            },
            sort: get_sort(a.sort_by, a.sort_order),
            include_header: a.include_header,
        }
    }
}

fn get_sort(sort_by: String, sort_order: String) -> Option<Sort> {
    if let Err(e) = SortBy::from_str(&sort_by) {
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
    src: String,
    src_type: SourceType,
    src_account: String,
    dst_format: String,
    src_file: String,
    dst_file: String,
    sort_by: String,
    sort_order: String,
    include_header: bool,
}

impl Args {
    fn new() -> Args {
        Args {
            src: Default::default(),
            src_type: SourceType::File,
            src_account: Default::default(),
            dst_format: Default::default(),
            src_file: Default::default(),
            dst_file: Default::default(),
            sort_by: Default::default(),
            sort_order: Default::default(),
            include_header: true,
        }
    }
}

pub fn parse_args(src_formats: Vec<&String>, dst_formats: Vec<&String>) -> Arguments {
    let mut args = Args::new();
    let src_options = format!("Source account. One of [{}]", src_formats.iter().sorted().iter().join(", "));
    let src_type_options = format!("Source type.");
    let dst_options = format!("Destination format. One of [{}]", dst_formats.iter().sorted().iter().join(", "));
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Transaction processor");

        ap.refer(&mut args.src)
            .add_option(&["-s", "--src"],
                        Store,
                        &src_options)
            .required();

        ap.refer(&mut args.src_type)
            .add_option(&["-t", "--src-type"],
                        Store,
                        &src_type_options)
            .required();

        ap.refer(&mut args.src_account)
            .add_option(&["-a", "--src-account"],
                        Store,
                        "Name of the account");

        ap.refer(&mut args.dst_format)
            .add_option(&["-d", "--dst-format"],
                        Store,
                        &dst_options)
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
                        "What to sort the output by");

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

        ap.parse_args_or_exit();
    }
    Arguments::from(args)
}