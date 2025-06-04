use crate::config::{Config, MatcherType};
use crate::transaction::Transaction;
use crate::util::currency_to_string_without_delim;
use regex::RegexBuilder;

#[derive(Debug)]
pub struct PayeeNormalizer {}

impl PayeeNormalizer {
    pub fn normalized_payee_id(config: &Config, s: &str) -> Option<String> {
        for (payee_id, payee) in &config.account().payees {
            for normalizer in &payee.normalizers {
                match &normalizer.normalizer_type {
                    MatcherType::Exact { exact_match_string } => {
                        let cmp_string = PayeeNormalizer::maybe_to_lower(normalizer.ignore_case, s);
                        let exact_match_string = PayeeNormalizer::maybe_to_lower(
                            normalizer.ignore_case,
                            exact_match_string,
                        );
                        if exact_match_string == cmp_string {
                            return Option::Some(payee_id.to_owned());
                        }
                    }
                    MatcherType::Contains { contains_string } => {
                        let cmp_string = PayeeNormalizer::maybe_to_lower(normalizer.ignore_case, s);
                        let contains_string = PayeeNormalizer::maybe_to_lower(
                            normalizer.ignore_case,
                            contains_string,
                        );
                        if cmp_string.contains(&contains_string) {
                            return Option::Some(payee_id.to_owned());
                        }
                    }
                    MatcherType::Regex { regex_string } => {
                        let re = RegexBuilder::new(regex_string)
                            .case_insensitive(normalizer.ignore_case)
                            .build()
                            .unwrap_or_else(|_| panic!("[{}] is not a valid regex", regex_string));
                        if re.is_match(s) {
                            return Option::Some(payee_id.to_owned());
                        }
                    }
                }
            }
        }
        println!("Payee was not normalized: {}", s);
        Option::None
    }

    fn maybe_to_lower(ignore_case: bool, s: &str) -> String {
        if ignore_case {
            s.to_lowercase()
        } else {
            s.to_owned()
        }
    }

    pub fn category_for_transaction(config: &Config, transaction: &Transaction) -> Option<String> {
        transaction
            .normalized_payee_id
            .as_ref()
            .and_then(|p| config.account().payees.get(p))
            .and_then(|p| p.category_ids.as_ref())
            .and_then(|c| {
                if c.is_empty() {
                    return Option::None;
                }
                if c.len() == 1 {
                    return c.first();
                }
                if !config.skip_prompts() {
                    PayeeNormalizer::prompt_select_category_id(config, transaction, c)
                } else {
                    Option::None
                }
            })
            .and_then(|x| config.category(x))
            .map(|c| c.name.to_owned())
    }

    fn prompt_select_category_id<'a>(
        config: &'a Config,
        transaction: &Transaction,
        category_ids: &'a [String],
    ) -> Option<&'a String> {
        println!();
        println!(
            "Multiple categories available for transaction: [payee: {}], [amount: {}], [type: {:?}], [date: {}], [raw payee: {}], [memo: {:?}], [status: {:?}]",
            transaction.payee(),
            currency_to_string_without_delim(&transaction.amount),
            transaction.transaction_type,
            transaction.date,
            transaction.raw_payee_name,
            transaction.memo,
            transaction.status
        );
        println!("Please select an option:");

        println!("{}. (skip)", 0);
        for (i, category_id) in category_ids.iter().enumerate() {
            if let Option::Some(c) = config.category(category_id) {
                println!("{}. {}", i + 1, c.name);
            }
        }
        let num: usize = read!();
        if num == 0 {
            Option::None
        } else {
            category_ids.get(num - 1)
        }
    }
}
