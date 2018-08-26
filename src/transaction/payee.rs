use regex::Regex;
use serde_json;
use std::io;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PayeeNormalizer {
    accounts: HashMap<String, AccountConfig>,
    categories: HashMap<String, Category>,
}

impl PayeeNormalizer {
    pub fn from_reader(r: Box<io::Read>) -> PayeeNormalizer {
        let c: PayeeNormalizeConfigInternal = serde_json::from_reader(r).unwrap();

        let mut accounts = HashMap::new();
        for account in c.accounts {
            accounts.insert(account.id.to_owned(), account.into());
        }

        let mut categories = HashMap::new();
        for category in c.categories {
            categories.insert(category.id.to_owned(), category.into());
        }

        PayeeNormalizer {
            accounts,
            categories,
        }
    }

    pub fn normalized_payee_id(&self, account_id: Option<String>, s: &str) -> Option<String> {
        account_id.as_ref().and_then(|x| {
            self.accounts.get(x)
        }).and_then(|x| {
            for n in &x.normalizers {
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
                            return Option::Some(n.payee_id.to_owned());
                        }
                    },
                    PayeeNormalizeType::Contains(match_string) => {
                        if cmp_string.contains(match_string) {
                            return Option::Some(n.payee_id.to_owned());
                        }
                    },
                    PayeeNormalizeType::Regex(re) => {
                        if re.is_match(&cmp_string) {
                            return Option::Some(n.payee_id.to_owned());
                        }
                    }
                }
            }
            println!("Payee '{}' was not normalized.", s);
            Option::None
        })
    }

    pub fn payee(&self, account_id: &str, payee_id: &str) -> Option<&Payee> {
        self.accounts.get(account_id).and_then(|x| {
            x.payees.get(payee_id)
        })
    }

    pub fn category_for_payee(&self, account_id: Option<String>, payee_id: &str) -> Option<String> {
        account_id.and_then(|x| {
            self.accounts.get(&x)
        }).and_then(|x| {
            x.payees.get(payee_id)
        }).and_then(|x| {
            x.category_id.as_ref()
        }).and_then(|x| {
            self.categories.get(x)
        }).and_then(|x| {
            Option::Some(x.name.to_owned())
        })
    }
}

#[derive(Debug)]
struct AccountConfig {
    id: String,
    name: String,
    normalizers: Vec<PayeeNormalizeItem>,
    payees: HashMap<String, Payee>,
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
pub struct Payee {
    pub id: String,
    pub name: String,
    pub category_id: Option<String>,
}

#[derive(Debug)]
struct Category {
    id: String,
    name: String,
}

impl From<AccountConfigInternal> for AccountConfig {
    fn from(a: AccountConfigInternal) -> Self {
        let mut normalizers = Vec::new();
        for normalizer in a.normalizers {
            normalizers.push(normalizer.into())
        }

        let mut payees = HashMap::new();
        for payee in a.payees {
            payees.insert(payee.id.to_owned(), payee.into());
        }
        AccountConfig {
            id: a.id,
            name: a.name,
            normalizers,
            payees,
        }
    }
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
            category_id: p.category_id,
        }
    }
}

impl From<CategoryInternal> for Category {
    fn from(c: CategoryInternal) -> Self {
        Category {
            id: c.id,
            name: c.name,
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
    #[serde(rename = "accounts")]
    accounts: Vec<AccountConfigInternal>,
    #[serde(rename = "categories")]
    categories: Vec<CategoryInternal>,
}

#[derive(Debug, Deserialize)]
struct AccountConfigInternal {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "name")]
    name: String,
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
    #[serde(rename = "categoryId")]
    category_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CategoryInternal {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "name")]
    name: String,
}