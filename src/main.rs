pub(crate) mod core;

mod accounts;
mod engine;
mod ledger;

use crate::core::{Accounts, Ledger, PaymentEngine, Tx};

use csv::Reader;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = PaymentEngine::default();
    let mut ledger = Ledger::default();
    let mut accounts = Accounts::default();
    let csv = fs::read_to_string("tests/test_cases/in2.csv")
        .unwrap()
        .replace(" ", "");
    let mut stream = Reader::from_reader(csv.as_bytes()); //= get_transaction_stream();
    for tx in stream.deserialize() {
        let tx: Tx = tx?;
        engine
            .process_transaction(&tx, &mut ledger, &mut accounts)
            .map_err(|e| eprintln!("{} (tx = {})", e.to_string(), &tx))
            .ok();
    }
    let csv_report = accounts.report_accounts_balances()?;
    output(csv_report);
    Ok(())
}

fn output(csv: String) {
    println!("{}", csv);
}
