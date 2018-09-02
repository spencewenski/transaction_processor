use transaction::Transaction;
use arguments::Arguments;
use config::{Config, MatcherType};
use std::collections::HashSet;

#[derive(Debug)]
pub struct PayeeNormalizer {
}

impl PayeeNormalizer {
    pub fn normalized_payee_id(config: Option<&Config>, account_id: Option<String>, s: &str) -> Option<String> {
        config.and_then(|c| {
            account_id.as_ref().and_then(|a| {
                c.accounts.get(a)
            })
        }).and_then(|x| {
            for n in &x.payee_normalizers {
                let cmp_string = {
                    if n.ignore_case {
                        s.to_lowercase()
                    } else {
                        s.to_owned()
                    }
                };
                match &n.normalizer_type {
                    MatcherType::Exact {exact_match_string} => {
                        if exact_match_string == &cmp_string {
                            return Option::Some(n.payee_id.to_owned());
                        }
                    },
                    MatcherType::Contains {contains_string} => {
                        if cmp_string.contains(contains_string) {
                            return Option::Some(n.payee_id.to_owned());
                        }
                    },
                    MatcherType::Regex {regex_string} => {
                        if regex_string.is_match(&cmp_string) {
                            return Option::Some(n.payee_id.to_owned());
                        }
                    }
                }
            }
            println!("Payee '{}' was not normalized.", s);
            Option::None
        })
    }

    pub fn category_for_transaction(args: &Arguments, config: Option<&Config>, account_id: Option<String>, transaction: &Transaction) -> Option<String> {
        if let Option::None = transaction.normalized_payee_id {
            return Option::None;
        }
        config.and_then(|c| {
            account_id.as_ref().and_then(|a| {
                c.accounts.get(a)
            })
        }).and_then(|a| {
            transaction.normalized_payee_id.as_ref().and_then(|p| {
                a.payees.get(p)
            })
        }).and_then(|p| {
            p.category_ids.as_ref()
        }).and_then(|c| {
            if c.len() == 0 {
                return Option::None;
            }
            if c.len() == 1 {
                return c.iter().last();
            }
            return PayeeNormalizer::prompt_select_category_id(args, config, transaction, c)
        }).and_then(|x| {
            config.and_then(|c| {
                c.categories.get(x)
            })
        }).and_then(|c| {
            Option::Some(c.name.to_owned())
        })
    }

    fn prompt_select_category_id<'a>(args: &Arguments, config: Option<&'a Config>, transaction: &Transaction, category_ids: &'a HashSet<String>) -> Option<&'a String> {
        let skip_prompts = args.skip_prompts || config.and_then(|x| {
            Option::Some(x.skip_prompts)
        }).unwrap_or(false);
        if skip_prompts {
            return Option::None
        }
        config.and_then(|c| {
            println!();
            println!("Multiple categories available for transaction [payee: {}], [amount: {}], [date: {}]. Please select an option:",
                     transaction.payee(), transaction.amount, transaction.date);

            println!("{}. {}", 0, "(skip)");
            let mut ids = Vec::new();
            for (i, category_id) in category_ids.iter().enumerate() {
                if let Option::Some(c) = c.categories.get(category_id) {
                    ids.push(c.id.to_owned());
                    println!("{}. {}", i + 1, c.name);
                }
            }
            let ids = ids;
            let num: usize = read!();
            if num == 0 {
                return Option::None
            }
            ids.get(num - 1).and_then(|x| {
                c.categories.get(x).and_then(|c| {
                    Option::Some(&c.id)
                })
            })
        })
    }
}
