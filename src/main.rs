#![allow(dead_code, unused_variables)]

use std::collections::HashMap;

enum TxType {
    Chargeback,
    Resolve,
    Dispute,
    Withdrawal,
    Deposit,
}

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

struct Tx {
    id: u32,
    client_id: u16,
    amount: f32, // optional
    tx_type: TxType,
    tx_state: Option<TxState>,
}

fn main() {}
