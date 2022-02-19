#![allow(dead_code, unused_variables)]

use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum TxType {
    Chargeback,
    Resolve,
    Dispute,
    Withdrawal,
    Deposit,
}

#[derive(Deserialize, Debug)]
enum TxState {
    Disputed,
    Resolved,
    Chargebacked,
}

struct Client {
    id: u16,
    transaction_history: HashMap<u32, Tx>,
    locked: bool,
    available: u16,
    held: u16,
    total: u16, //?
}

#[derive(Deserialize, Debug)]
struct Tx {
    #[serde(alias = "type")]
    tx_type: TxType,
    #[serde(alias = "client")]
    client_id: u16,
    #[serde(alias = "tx")]
    id: u32,
    amount: f32,               // optional
    tx_state: Option<TxState>, // optional
}

fn main() -> Result<(), csv::Error> {
    let clients: Vec<Client> = vec![];
    let transactions: HashMap<u32, Tx> = HashMap::new();
    let csv = fs::read_to_string("tests/test_cases/in1.csv")?.replace(" ", "");
    let mut reader = Reader::from_reader(csv.as_bytes());
    for record in reader.deserialize() {
        let tx: Tx = record?;
        println!("{:?}", tx);
    }

    Ok(())
}

fn get_transaction_stream() -> Tx {
    todo!()
}
