use regex::Regex;
use serde_json;
use std::io;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PayeeNormalizer {
    normalizers: Vec<PayeeNormalizeItem>,
    payees: HashMap<String, Payee>,
}

impl PayeeNormalizer {
    pub fn from_reader(r: Box<io::Read>) -> PayeeNormalizer {
        let c: PayeeNormalizeConfigInternal = serde_json::from_reader(r).unwrap();

        let mut normalizers = Vec::new();
        for normalizer in c.normalizers {
            normalizers.push(normalizer.into())
        }

        let mut payees = HashMap::new();
        for payee in c.payees {
            payees.insert(payee.id.to_owned(), payee.into());
        }

        PayeeNormalizer {
            normalizers,
            payees,
        }
    }

    pub fn normalize_str(&self, s: &str) -> String {
        for n in &self.normalizers {
            let cmp_string = {
                if n.ignore_case {
                    s.to_lowercase()
                } else {
                    s.to_owned()
                }
            };
            match &n.normalize_type {
                PayeeNormalizeType::Exact(match_string) => {
                    if match_string == &cmp_string {
                        return self.get_normalized_name(&n.payee_id, s).to_owned();
                    }
                },
                PayeeNormalizeType::Contains(match_string) => {
                    if cmp_string.contains(match_string) {
                        return self.get_normalized_name(&n.payee_id, s).to_owned();
                    }
                },
                PayeeNormalizeType::Regex(re) => {
                    if re.is_match(&cmp_string) {
                        return self.get_normalized_name(&n.payee_id, s).to_owned();
                    }
                }
            }
        }
        println!("Payee '{}' was not normalized.", s);
        s.to_owned()
    }

    // Get the normalized name of the payee, or the raw payee name if the payee does not exist
    fn get_normalized_name<'a>(&'a self, id: &'a str, raw_name: &'a str) -> &'a str {
         if let Option::Some(p) = self.payees.get(id) {
             return &p.name;
         } else {
             return raw_name;
         }
    }
}

#[derive(Debug)]
enum PayeeNormalizeType {
    Exact(String),
    Contains(String),
    Regex(Regex),
}

#[derive(Debug)]
struct PayeeNormalizeItem {
    normalize_type: PayeeNormalizeType,
    payee_id: String,
    ignore_case: bool,
}

#[derive(Debug)]
struct Payee {
    id: String,
    name: String,
}

impl From<PayeeNormalizeItemInternal> for PayeeNormalizeItem {
    fn from(p: PayeeNormalizeItemInternal) -> Self {

        let t = match p.normalize_type {
            PayeeNormalizeTypeInternal::Exact => PayeeNormalizeType::Exact(p.normalize_match_string),
            PayeeNormalizeTypeInternal::Contains => PayeeNormalizeType::Contains(p.normalize_match_string),
            PayeeNormalizeTypeInternal::Regex => PayeeNormalizeType::Regex(Regex::new(&p.normalize_match_string).unwrap()),
        };
        PayeeNormalizeItem {
            normalize_type: t,
            payee_id: p.payee_id,
            ignore_case: p.ignore_case.unwrap_or(true)
        }
    }
}

impl From<PayeeInternal> for Payee {
    fn from(p: PayeeInternal) -> Self {
        Payee {
            id: p.id,
            name: p.name,
        }
    }
}

#[derive(Debug, Deserialize)]
enum PayeeNormalizeTypeInternal {
    Exact,
    Contains,
    Regex,
}

#[derive(Debug, Deserialize)]
struct PayeeNormalizeItemInternal {
    #[serde(rename = "type")]
    normalize_type: PayeeNormalizeTypeInternal,
    #[serde(rename = "matchString")]
    normalize_match_string: String,
    #[serde(rename = "payeeId")]
    payee_id: String,
    #[serde(rename = "ignoreCase")]
    ignore_case: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PayeeNormalizeConfigInternal {
    #[serde(rename = "normalizers")]
    normalizers: Vec<PayeeNormalizeItemInternal>,
    #[serde(rename = "payees")]
    payees: Vec<PayeeInternal>,
}

#[derive(Debug, Deserialize)]
struct PayeeInternal {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "name")]
    name: String,
}