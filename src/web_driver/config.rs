use std::path::{Path};
use tempfile::{TempDir};

// Configuration
pub struct WebDriverConfig {
    download_dir: TempDir,
    auto_download: AutoDownload,
    args: Vec<String>,
    address: String,
}

impl WebDriverConfig {
    pub fn build(auto_download: AutoDownload,
                 headless: bool,
                 address: &str) -> WebDriverConfig {
        Self::build_with_args(auto_download, headless, address, Vec::new())
    }

    pub fn build_with_args(auto_download: AutoDownload,
                           headless: bool,
                           address: &str,
                           args: Vec<String>) -> WebDriverConfig {
        Self::build_all(auto_download, headless, address, args)
    }

    fn build_all(auto_download: AutoDownload,
                 headless: bool,
                 address: &str,
                 args: Vec<String>) -> WebDriverConfig {
        let mut args = args;
        if headless {
            args.push(String::from("-headless"));
        }

        WebDriverConfig {
            download_dir: TempDir::new().unwrap(),
            auto_download,
            args,
            address: String::from(address),
        }
    }

    pub fn add_arg(mut self, arg: String) -> WebDriverConfig {
        self.args.push(arg);
        self
    }

    pub fn get_auto_download(&self) -> &AutoDownload {
        &self.auto_download
    }

    pub fn get_download_path(&self) -> &Path {
        self.download_dir.path()
    }

    pub fn get_args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn get_address(&self) -> &String {
        &self.address
    }
}

pub enum AutoDownload {
    True(MimeType),
    False,
}

// Mime types
pub enum MimeType {
    TextCsv,
}

impl <'a> From<&'a MimeType> for String {
    fn from(m: &'a MimeType) -> Self {
        match m {
            &MimeType::TextCsv => String::from("text/csv"),
        }
    }
}