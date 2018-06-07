extern crate transaction_processor;

use transaction_processor::transaction::*;
use transaction_processor::transaction::formats::ally_bank::*;

fn main() {
    let ally = AllyTransaction::build_transaction(
        String::from("2018-06-01"),
        String::from("01:01:54"),
        String::from("-4874"),
        AllyTransactionType::Withdrawal,
        String::from("Internet transfer to Online Savings account XXXXXX5489"));
    println!("{:?}", ally);

    let transaction = Transaction::from(ally);
    println!("\n{:?}", transaction);

    let transaction = transaction.clean_payee(String::from("Ally Savings"));
    println!("\n{:?}", transaction);

    let transaction = transaction.update_category(String::from("Category A"));
    println!("\n{:?}", transaction);
}