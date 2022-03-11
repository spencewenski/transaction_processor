use anyhow::anyhow;
use currency::Currency;
use std::fs::File;
use std::io;

pub fn get_optional_string(s: &str) -> Option<String> {
    if s.trim().len() > 0 {
        Option::Some(s.to_owned())
    } else {
        Option::None
    }
}

pub fn reader_from_file_name(filename: &str) -> anyhow::Result<Box<dyn io::Read>> {
    match File::open(filename) {
        Ok(f) => Ok(Box::new(io::BufReader::new(f))),
        Err(e) => Err(anyhow!("Unable to open file [{}]: {}", filename, e)),
    }
}

// For some reason, the Currency type prepends a ',' to values in the hundreds, so just remove
// all ',' from the string generated by Currency to avoid such silliness.
pub fn currency_to_string_without_delim(c: &Currency) -> String {
    let s = c.to_string();
    s.replace(",", "")
}

#[cfg(test)]
mod test {
    use currency::Currency;
    use util::{currency_to_string_without_delim, get_optional_string};

    #[test]
    fn test_currency_to_string_without_delim() {
        let c = Currency::from_str("1,000,000").unwrap();
        let s = currency_to_string_without_delim(&c);
        assert_eq!(s, "1000000.00");

        let c = Currency::from_str("10").unwrap();
        let s = currency_to_string_without_delim(&c);
        assert_eq!(s, "10.00");
    }

    #[test]
    fn test_get_optional_string() {
        let s = "";
        let o = get_optional_string(s);
        assert_eq!(o, Option::None);

        let s = "   ";
        let o = get_optional_string(s);
        assert_eq!(o, Option::None);

        let s = "\tfoo ";
        let o = get_optional_string(s);
        assert_eq!(o, Option::Some(String::from("\tfoo ")))
    }
}
