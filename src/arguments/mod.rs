use argparse::{ArgumentParser, Store};
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
            }
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
        ap.parse_args_or_exit();
    }
    Arguments::from(args)
}