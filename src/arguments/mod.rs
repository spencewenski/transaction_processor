use argparse::{ArgumentParser, Store, StoreTrue, StoreFalse};
use itertools::Itertools;
use std::str::FromStr;
use std::io::{stdout, Write};
use rpassword;
use util;

#[derive(Debug)]
pub struct Arguments {
    pub src: String,
    pub src_type: SourceType,
    pub src_account: Option<Account>,
    pub dst_format: String,
    pub src_file: Option<String>,
    pub dst_file: Option<String>,
    pub sort: Option<Sort>,
    pub include_header: bool,
    pub ignore_pending: bool,
    pub normalize_config: Option<String>,
    pub skip_prompts: bool,
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
            src_account: get_account(&a.src_type, a.src_account, a.src_username),
            src_type: a.src_type,
            dst_format: a.dst_format,
            src_file: util::get_optional_string(a.src_file),
            dst_file: util::get_optional_string(a.dst_file),
            sort: get_sort(a.sort_by, a.sort_order),
            include_header: a.include_header,
            ignore_pending: a.ignore_pending,
            normalize_config: util::get_optional_string(a.normalize_config),
            skip_prompts: a.skip_prompts,
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

#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub credentials: Option<AccountCredentials>,
}

#[derive(Debug)]
pub struct AccountCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Default)]
struct AccountBuilder {
    name: String,
    username: Option<String>,
    password: Option<String>,
}

impl AccountBuilder {
    fn build(self) -> Account {
        Account {
            credentials: self.build_credentials(),
            name: self.name,
        }
    }

    fn name(&mut self, name: String) -> &mut AccountBuilder {
        self.name = name;
        self
    }

    fn username(&mut self, username: String) -> &mut AccountBuilder {
        self.username = Option::Some(username);
        self
    }

    fn password(&mut self, password: String) -> &mut AccountBuilder {
        self.password = Option::Some(password);
        self
    }

    fn build_credentials(&self) -> Option<AccountCredentials> {
        if let Option::None = self.username {
            return Option::None;
        } else if let Option::None = self.password {
            return Option::None;
        }
        Option::Some(AccountCredentials {
            username: self.username.to_owned().unwrap(),
            password: self.password.to_owned().unwrap(),
        })
    }
}

impl ToOwned for Account {
    type Owned = Account;

    fn to_owned(&self) -> <Self as ToOwned>::Owned {
        Account {
            name: self.name.to_owned(),
            credentials: if let Option::Some(ref c) = self.credentials {
                Option::Some(AccountCredentials {
                    username: c.username.to_owned(),
                    password: c.password.to_owned(),
                })
            } else {
                Option::None
            }
        }
    }
}

fn get_account(source_type: &SourceType, name: String, username: String) -> Option<Account> {
    if name.len() == 0 {
        return Option::None;
    }
    let mut builder = AccountBuilder::default();
    builder.name(name);

    if let SourceType::File = source_type {
        return Option::Some(builder.build());
    }

    let username = {
        if username.len() > 0 {
            username
        } else {
            println!();
            print!("Username: ");
            stdout().flush().unwrap();
            read!()
        }
    };
    builder.username(username);

    let password = {
        println!();
        rpassword::prompt_password_stdout("Password: ").unwrap()
    };
    builder.password(password);

    Option::Some(builder.build())
}

struct Args {
    src: String,
    src_type: SourceType,
    src_account: String,
    src_username: String,
    dst_format: String,
    src_file: String,
    dst_file: String,
    sort_by: String,
    sort_order: String,
    include_header: bool,
    ignore_pending: bool,
    normalize_config: String,
    skip_prompts: bool,
}

impl Args {
    fn new() -> Args {
        Args {
            src: Default::default(),
            src_type: SourceType::File,
            src_account: Default::default(),
            src_username: Default::default(),
            dst_format: Default::default(),
            src_file: Default::default(),
            dst_file: Default::default(),
            sort_by: Default::default(),
            sort_order: Default::default(),
            include_header: true,
            ignore_pending: false,
            normalize_config: Default::default(),
            skip_prompts: false,
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

        ap.refer(&mut args.src_username)
            .add_option(&["--src-username"],
                        Store,
                        "Username for the source account.");

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

        ap.refer(&mut args.ignore_pending)
            .add_option(&["--ignore-pending"],
                        StoreTrue,
                        "Ignore pending transactions. Defaults to false");

        ap.refer(&mut args.normalize_config)
            .add_option(&["--normalize-config"],
                        Store,
                        "Name of the normalization config file.");

        ap.refer(&mut args.skip_prompts)
            .add_option(&["--skip-prompts"],
                        StoreTrue,
                        "Skip any prompts for optional user input.");

        ap.parse_args_or_exit();
    }
    Arguments::from(args)
}