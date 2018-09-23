use std::collections::HashSet;
use std::path::{Path};
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io;

pub fn get_files_in_dir(dir: &Path) -> HashSet<OsString> {
    let mut ignore_extensions = HashSet::new();
    ignore_extensions.insert(String::from("part"));
    get_files_ignore_ext(dir, &ignore_extensions)
}

pub fn get_files_ignore_ext(dir: &Path, ignore_extensions: &HashSet<String>) -> HashSet<OsString> {
    let mut files = HashSet::new();
    let paths = fs::read_dir(dir).unwrap();
    for path in paths {
        if let Ok(p) = path {
            let p = p.path();
            if p.is_file() && !ignore_extensions.contains(p.extension().unwrap().to_str().unwrap()) {
                files.insert(p.into());
            }
        }
    }
    files
}

pub fn get_optional_string(s: String) -> Option<String> {
    if s.trim().len() != 0 {
        Option::Some(s)
    } else {
        Option::None
    }
}

pub fn reader_from_file_name(f: &str) -> Box<io::Read> {
    let f = File::open(f).expect("File not found");
    Box::new(io::BufReader::new(f))
}