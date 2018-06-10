use argparse::{ArgumentParser, Store};

#[derive(Debug, Default)]
pub struct Arguments {
    pub src_format: String,
    pub dst_format: String,
    pub src_file: Option<String>,
    pub dst_file: Option<String>,
}

impl From<Args> for Arguments {
    fn from(a: Args) -> Arguments {
        Arguments {
            src_format: a.src_format,
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

#[derive(Default)]
struct Args {
    src_format: String,
    dst_format: String,
    src_file: String,
    dst_file: String,
}

pub fn parse_args() -> Arguments {
    let mut args : Args = Default::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Transaction processor");
        ap.refer(&mut args.src_format)
            .add_option(&["-s", "--src-format"],
                        Store,
                        "Source format")
            .required();
        ap.refer(&mut args.dst_format)
            .add_option(&["-d", "--dst-format"],
                        Store,
                        "Destination format")
            .required();
        ap.refer(&mut args.src_file)
            .add_option(&["-i", "--src-file"],
                        Store,
                        "Source file");
        ap.refer(&mut args.dst_file)
            .add_option(&["-o", "--dst-file"],
                        Store,
                        "Destination file");
        ap.parse_args_or_exit()
    }
    Arguments::from(args)
}
