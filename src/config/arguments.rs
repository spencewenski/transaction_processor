use super::{SortBy, SortOrder};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Arguments {
    #[clap(long, value_parser, value_name = "FILE")]
    pub account_config_file: PathBuf,
    #[clap(long, value_parser, value_name = "FILE")]
    pub categories_config_file: PathBuf,
    #[clap(long, value_parser, value_name = "FILE")]
    pub src_format_config_file: PathBuf,
    #[clap(long, value_parser, value_name = "FILE")]
    pub dst_format_config_file: PathBuf,
    #[clap(short = 'i', long, value_parser, value_name = "FILE")]
    pub src_file: Option<PathBuf>,
    #[clap(short = 'o', long, value_parser, value_name = "FILE")]
    pub dst_file: Option<PathBuf>,
    #[clap(long, value_enum)]
    pub sort_by: Option<SortBy>,
    #[clap(long, value_enum)]
    pub sort_order: Option<SortOrder>,
    #[clap(long)]
    pub include_header: Option<bool>,
    #[clap(long)]
    pub ignore_pending: Option<bool>,
    #[clap(long)]
    pub skip_prompts: Option<bool>,
}
