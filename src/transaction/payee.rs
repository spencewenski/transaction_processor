use transaction::Transaction;
use arguments::Arguments;
use config::{Config, MatcherType};
use regex::RegexBuilder;

#[derive(Debug)]
pub struct PayeeNormalizer {
}

impl PayeeNormalizer {
    pub fn normalized_payee_id(config: &Config, account_id: &str, s: &str) -> Option<String> {
        config.accounts.get(account_id).and_then(|x| {
            for n in &x.payee_normalizers {
                match &n.normalizer_type {
                    MatcherType::Exact {exact_match_string} => {
                        let cmp_string = PayeeNormalizer::maybe_to_lower(n.ignore_case, s);
                        let exact_match_string = PayeeNormalizer::maybe_to_lower(n.ignore_case, exact_match_string);
                        if exact_match_string == cmp_string {
                            return Option::Some(n.payee_id.to_owned());
                        }
                    },
                    MatcherType::Contains {contains_string} => {
                        let cmp_string = PayeeNormalizer::maybe_to_lower(n.ignore_case, s);
                        let contains_string = PayeeNormalizer::maybe_to_lower(n.ignore_case, contains_string);
                        if cmp_string.contains(&contains_string) {
                            return Option::Some(n.payee_id.to_owned());
                        }
                    },
                    MatcherType::Regex {regex_string} => {
                        let re = RegexBuilder::new(regex_string)
                            .case_insensitive(n.ignore_case)
                            .build()
                            .expect("Invalid regex");
                        if re.is_match(s) {
                            return Option::Some(n.payee_id.to_owned());
                        }
                    }
                }
            }
            println!("Payee was not normalized: {}", s);
            Option::None
        })
    }

    fn maybe_to_lower(ignore_case: bool, s: &str) -> String {
        if ignore_case {
            s.to_lowercase()
        } else {
            s.to_owned()
        }
    }

    pub fn category_for_transaction(args: &Arguments, config: &Config, account_id: &str, transaction: &Transaction) -> Option<String> {
        if let Option::None = transaction.normalized_payee_id {
            return Option::None;
        }
        config.accounts.get(account_id).and_then(|a| {
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
                return c.first()
            }
            return PayeeNormalizer::prompt_select_category_id(args, config, transaction, c)
        }).and_then(|x| {
            config.categories.get(x)
        }).and_then(|c| {
            Option::Some(c.name.to_owned())
        })
    }

    fn prompt_select_category_id<'a>(args: &Arguments, config: &'a Config, transaction: &Transaction, category_ids: &'a Vec<String>) -> Option<&'a String> {
        if args.skip_prompts || config.skip_prompts {
            return Option::None
        }
        println!();
        println!("Multiple categories available for transaction: [payee: {}], [amount: {}], [date: {}]",
                 transaction.payee(), transaction.amount, transaction.date);
        println!("Please select an option:");

        println!("{}. {}", 0, "(skip)");
        for (i, category_id) in category_ids.iter().enumerate() {
            if let Option::Some(c) = config.categories.get(category_id) {
                println!("{}. {}", i + 1, c.name);
            }
        }
        let num: usize = read!();
        if num == 0 {
            return Option::None
        } else {
            category_ids.get(num - 1)
        }
    }
}
