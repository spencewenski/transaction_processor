use argparse::{ArgumentParser, Store};

#[derive(Debug, Default)]
pub struct Arguments {
    pub command: String,
    pub src_format: String,
    pub dst_format: String,
    pub src_file: String,
    pub dst_file: String,
}

pub fn parse_args() -> Arguments {
    let mut args : Arguments = Default::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Transaction processor");
        ap.refer(&mut args.src_format)
            .add_option(&["-s", "--src-format"],
                        Store,
                        "Source format");
        ap.refer(&mut args.dst_format)
            .add_option(&["-d", "--dst-format"],
                        Store,
                        "Destination format");
        ap.refer(&mut args.src_file)
            .add_option(&["-i", "--src-file"],
                        Store,
                        "Source file");
        ap.refer(&mut args.dst_file)
            .add_option(&["-o", "--dst-file"],
                        Store,
                        "Destination file");
        ap.refer(&mut args.command)
            .add_argument("Command",
                          Store,
                          "Command to run");
        ap.parse_args_or_exit()
    }
    args
}
