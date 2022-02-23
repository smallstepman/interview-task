mod utils; // importing as a first item to register the macro before everything else

pub(crate) mod core;

mod accounts;
mod engine;
mod ledger;

use crate::core::{Accounts, Ledger, PaymentEngine, Tx};

use clap::{self, Arg, ArgMatches};
use csv::{Reader, ReaderBuilder, Trim};
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let csv_path = input();
    let mut engine = PaymentEngine::default();
    let mut ledger = Ledger::default();
    let mut accounts = Accounts::default();
    let mut transactions = get_transactions(csv_path.value_of("csv-path"))?;
    for tx in transactions.deserialize() {
        let tx: Tx<ledger::transaction::Default> = tx?;
        engine
            .process_transaction(tx, &mut ledger, &mut accounts)
            .map_err(|e| eprintln!("{} (tx = {})", e.to_string(), &tx))
            .ok();
    }
    let csv_report = accounts.report_accounts_balances()?;
    output(&csv_report);
    Ok(())
}

/// `stdin`, parsed with `clap`
fn input() -> ArgMatches {
    clap::Command::new("interviewpuzzle")
        .arg_required_else_help(true)
        .arg(
            Arg::new("csv-path")
                .help("Path to csv file.")
                .takes_value(true),
        )
        .get_matches()
}

/// Returns a type with `.deserialized()` method attached to it, which produces
/// an iterator of incoming transactions
fn get_transactions(csv_path: Option<&str>) -> csv::Result<Reader<File>> {
    let transactions = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(csv_path.unwrap())?;
    Ok(transactions)
}

/// `stdout`, but with a prettier name
fn output(csv: &str) {
    print!("{}", csv);
}
