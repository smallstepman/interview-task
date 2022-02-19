use csv::Reader;
use csv::Writer;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

type ClientId = u16;
type TransactionId = u32;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum TxType {
    Chargeback,
    Resolve,
    Dispute,
    Withdrawal,
    Deposit,
}

#[derive(Deserialize, Debug, Clone)]
enum TxState {
    Disputed,
    Resolved,
    Chargebacked,
}

#[derive(Serialize, Debug, Clone)]
struct Client {
    id: ClientId,
    // transaction_history: Vec<TransactionId>, // ?
    locked: bool,
    available: f32,
    held: f32,
    // total: u16, // ?
}

impl Client {
    fn new(id: ClientId) -> Self {
        Client {
            id,
            locked: false,
            available: 0.0,
            held: 0.0,
        }
    }
    fn freeze(&mut self) {
        self.locked = true;
    }
    fn deposit(&mut self, amount: f32) {
        self.available += amount;
    }
    fn withdrawal(&mut self, amount: f32) {
        self.available -= amount;
    }
    fn hold(&mut self, amount: f32) {
        self.available -= amount;
        self.held += amount;
    }
    fn resolve(&mut self, amount: f32) {
        self.held -= amount;
        self.available += amount;
    }
    fn chargeback(&mut self, amount: f32) {
        self.held -= amount;
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Tx {
    #[serde(alias = "type")]
    tx_type: TxType,
    #[serde(alias = "client")]
    client_id: ClientId,
    #[serde(alias = "tx")]
    id: TransactionId,
    amount: Option<f32>,       // optional
    tx_state: Option<TxState>, // optional
}

impl Tx {
    fn dispute(&mut self) {
        self.tx_state = Some(TxState::Disputed);
    }
    fn chargeback(&mut self) {
        self.tx_state = Some(TxState::Chargebacked);
    }
    fn resolve(&mut self) {
        self.tx_state = Some(TxState::Resolved);
    }
}

//

// fn get_transaction_stream() -> DeserializeRecordsIter<'staticu8> {
//     let csv = fs::read_to_string("tests/test_cases/in1.csv")
//         .unwrap()
//         .replace(" ", "");
//     let mut reader = Reader::from_reader(csv.as_bytes());
//     return reader.deserialize().into_iter();
// }

#[derive(Default)]
struct Ledger(HashMap<TransactionId, Tx>);

impl Ledger {
    fn get_transaction(&mut self, tx: &Tx) -> Option<&mut Tx> {
        self.0.get_mut(&tx.id)
    }
}

#[derive(Default)]
struct Accounts(HashMap<ClientId, Client>);

impl Accounts {
    // fn get_client(&mut self, tx: &Tx) -> &mut Client {
    fn get_client(&mut self, id: ClientId) -> &mut Client {
        self.0.entry(id).or_insert(Client::new(id))
    }

    pub fn report_accounts_balances(&self) -> Result<String, Box<dyn Error>> {
        let mut wtr = Writer::from_writer(vec![]);
        for client in self.0.values() {
            wtr.serialize(client)?;
        }
        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }
}

#[derive(Default)]
struct PaymentEngine;
impl PaymentEngine {
    pub fn process_transaction(&mut self, tx: Tx, ledger: &mut Ledger, clients: &mut Accounts) {
        let client = clients.get_client(tx.client_id);
        if let Some(existing_tx) = ledger.get_transaction(&tx) {
            match tx.tx_type {
                TxType::Dispute => {
                    client.hold(tx.amount.unwrap());
                    existing_tx.dispute();
                }
                TxType::Resolve => {
                    client.resolve(tx.amount.unwrap());
                    existing_tx.resolve();
                }
                TxType::Chargeback => {
                    client.freeze();
                    client.chargeback(tx.amount.unwrap());
                    existing_tx.chargeback();
                }
                TxType::Withdrawal | TxType::Deposit => panic!(),
            }
        } else {
            match tx.tx_type {
                TxType::Deposit => client.deposit(tx.amount.unwrap()),
                TxType::Withdrawal => client.withdrawal(tx.amount.unwrap()),
                TxType::Chargeback | TxType::Resolve | TxType::Dispute => panic!(),
            }
        }
    }
}

fn output(csv: String) {
    println!("{}", csv);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = PaymentEngine::default();
    let mut ledger = Ledger::default();
    let mut accounts = Accounts::default();
    let csv = fs::read_to_string("tests/test_cases/in1.csv")
        .unwrap()
        .replace(" ", "");
    let mut stream = Reader::from_reader(csv.as_bytes()); //= get_transaction_stream();
    for tx in stream.deserialize() {
        engine.process_transaction(tx?, &mut ledger, &mut accounts)
    }
    let csv_report = accounts.report_accounts_balances()?;
    output(csv_report);
    Ok(())
}
