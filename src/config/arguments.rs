use super::{SortBy, SortOrder};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Arguments {
    #[clap(long, parse(from_os_str), value_name = "FILE")]
    pub account_config_file: PathBuf,
    #[clap(long, parse(from_os_str), value_name = "FILE")]
    pub categories_config_file: PathBuf,
    #[clap(long, parse(from_os_str), value_name = "FILE")]
    pub src_format_config_file: PathBuf,
    #[clap(long, parse(from_os_str), value_name = "FILE")]
    pub dst_format_config_file: PathBuf,
    #[clap(short = 'i', long, parse(from_os_str), value_name = "FILE")]
    pub src_file: Option<PathBuf>,
    #[clap(short = 'o', long, parse(from_os_str), value_name = "FILE")]
    pub dst_file: Option<PathBuf>,
    #[clap(long, arg_enum)]
    pub sort_by: Option<SortBy>,
    #[clap(long, arg_enum)]
    pub sort_order: Option<SortOrder>,
    #[clap(long)]
    pub include_header: Option<bool>,
    #[clap(long)]
    pub ignore_pending: Option<bool>,
    #[clap(long)]
    pub skip_prompts: Option<bool>,
}
