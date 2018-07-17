pub mod config;

use future::FutureValue;
use util::*;
use std::ffi::OsString;
use webdriver::capabilities::Capabilities;
use std::path::{Path};
use rustc_serialize::json::{Json, ToJson};
use fantoccini::{Client};
use tokio_core;
use std::collections::HashSet;

pub fn create_core() -> tokio_core::reactor::Core {
    tokio_core::reactor::Core::new().unwrap()
}

pub fn create_client(core: &mut tokio_core::reactor::Core, config: &config::WebDriverConfig) -> Client {
    let client = Client::with_capabilities(config.get_address(),
                              create_gecko_caps(&config),
                              &core.handle());
    core.run(client).unwrap()
}

fn create_gecko_caps(config: &config::WebDriverConfig) -> Capabilities {
    let mut firefox_prefs = Capabilities::new();
    if let config::AutoDownload::True(t) = config.get_auto_download() {
        firefox_prefs.insert(String::from("browser.helperApps.neverAsk.saveToDisk"),
                             String::from(t).to_json());
        firefox_prefs.insert(String::from("browser.download.folderList"),
                             Json::I64(2));
    }
    firefox_prefs.insert(String::from("browser.download.dir"),
                         config.get_download_path().to_str().unwrap().to_json());

    let mut options = Capabilities::new();
    options.insert(String::from("args"), config.get_args().to_json());
    options.insert(String::from("prefs"), firefox_prefs.to_json());

    let mut body = Capabilities::new();
    body.insert(String::from("moz:firefoxOptions"), options.to_json());
    body
}

pub fn wait_for_new_files(client: &mut Client,
                          dir: &Path,
                          starting_files: &HashSet<OsString>) -> FutureValue<HashSet<OsString>> {
    let mut val = FutureValue::new();
    client.wait_for( |_| {
        let files = get_files_in_dir(dir);
        let diff = files.difference(&starting_files);
        let mut new_files = HashSet::new();
        for file in diff {
            new_files.insert(file.to_owned());
        }
        let new_files = new_files;
        if new_files.len() > 0 {
            val.set(new_files);
            return true;
        }
        return false;
    });
    val
}
