use csv::Reader;
use csv::Writer;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::{error::Error, fmt};

type ClientId = u16;
type TransactionId = u32;

#[derive(Debug)]
struct NotEnoughFunds;
impl Error for NotEnoughFunds {}
impl fmt::Display for NotEnoughFunds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Not enough funds in client's account.")
    }
}

#[derive(Debug)]
struct AccountLocked;
impl Error for AccountLocked {}
impl fmt::Display for AccountLocked {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to modify locked account.")
    }
}

#[derive(Debug)]
struct DuplicateTransaction;
impl Error for DuplicateTransaction {}
impl fmt::Display for DuplicateTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Deposit or Withdrawal transaction with the same ID already exists in the ledger."
        )
    }
}

#[derive(Debug)]
struct NonExistingTransaction;
impl Error for NonExistingTransaction {}
impl fmt::Display for NonExistingTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attempted to postprocess a non existent transaction.")
    }
}

#[derive(Debug)]
struct IrreversableTransaction;
impl Error for IrreversableTransaction {}
impl fmt::Display for IrreversableTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Attempted to chargeback a transaction which is currently not disputed."
        )
    }
}

#[derive(Debug)]
struct UnresolvableTransaction;
impl Error for UnresolvableTransaction {}
impl fmt::Display for UnresolvableTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Attempted to resolve a transaction which is currently not disputed."
        )
    }
}

#[derive(Debug)]
struct UndisputableTransaction;
impl Error for UndisputableTransaction {}
impl fmt::Display for UndisputableTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Withdrawal transaction cannot be disputed.")
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
enum TxType {
    Chargeback,
    Resolve,
    Dispute,
    Withdrawal,
    Deposit,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
enum TxState {
    Disputed,
    Resolved,
    Chargebacked,
}

#[derive(Serialize, Debug, Clone)]
struct Client {
    id: ClientId,
    locked: bool,
    available: f32,
    held: f32,
}

type ClientActionResult = Result<(), Box<dyn Error>>;
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
    fn deposit(&mut self, amount: f32) -> ClientActionResult {
        self.is_unlocked()?;
        self.available += amount;
        Ok(())
    }
    fn withdrawal(&mut self, amount: f32) -> ClientActionResult {
        self.is_unlocked()?;
        self.has_enough_funds(amount)?;
        self.available -= amount;
        Ok(())
    }
    fn hold(&mut self, amount: f32) -> ClientActionResult {
        self.is_unlocked()?;
        self.has_enough_funds(amount)?;
        self.available -= amount;
        self.held += amount;
        Ok(())
    }
    fn resolve(&mut self, amount: f32) -> ClientActionResult {
        self.is_unlocked()?;
        self.held -= amount;
        self.available += amount;
        Ok(())
    }
    fn chargeback(&mut self, amount: f32) -> ClientActionResult {
        self.held -= amount;
        Ok(())
    }
    fn is_unlocked(&mut self) -> Result<(), AccountLocked> {
        if !self.locked {
            Ok(())
        } else {
            Err(AccountLocked)
        }
    }
    fn has_enough_funds(&mut self, amount: f32) -> Result<(), NotEnoughFunds> {
        if self.available >= amount {
            Ok(())
        } else {
            Err(NotEnoughFunds)
        }
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

impl Display for Tx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "type:{:?},client:{},tx:{},amount:{:?},state:{:?}",
            self.tx_type, self.client_id, self.id, self.amount, self.tx_state
        )
    }
}
impl Tx {
    fn dispute(&mut self) -> Result<(), UndisputableTransaction> {
        if self.tx_type != TxType::Withdrawal
            && (self.tx_state == Some(TxState::Resolved) || self.tx_state.is_none())
        {
            self.tx_state = Some(TxState::Disputed);
            Ok(())
        } else {
            Err(UndisputableTransaction)
        }
    }
    fn chargeback(&mut self) -> Result<(), IrreversableTransaction> {
        if let Some(TxState::Disputed) = self.tx_state {
            self.tx_state = Some(TxState::Chargebacked);
            Ok(())
        } else {
            Err(IrreversableTransaction)
        }
    }
    fn resolve(&mut self) -> Result<(), UnresolvableTransaction> {
        if Some(TxState::Disputed) == self.tx_state {
            self.tx_state = Some(TxState::Resolved);
            Ok(())
        } else {
            Err(UnresolvableTransaction)
        }
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

#[derive(Default, Debug)]
struct Ledger(HashMap<TransactionId, Tx>);

impl Ledger {
    fn get_transaction(&mut self, tx: &Tx) -> Option<&mut Tx> {
        self.0.get_mut(&tx.id)
    }
    fn insert_transaction(&mut self, tx: &Tx) {
        self.0.insert(tx.id, tx.clone());
    }
}

#[derive(Default, Debug)]
struct Accounts(HashMap<ClientId, Client>);

impl Accounts {
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
    pub fn process_transaction(
        &mut self,
        tx: &Tx,
        ledger: &mut Ledger,
        clients: &mut Accounts,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(existing_tx) = ledger.get_transaction(&tx) {
            match tx.tx_type {
                TxType::Dispute => {
                    existing_tx.dispute()?;
                    clients
                        .get_client(tx.client_id)
                        .hold(existing_tx.amount.unwrap())?;
                }
                TxType::Resolve => {
                    existing_tx.resolve()?;
                    clients
                        .get_client(tx.client_id)
                        .resolve(existing_tx.amount.unwrap())?;
                }
                TxType::Chargeback => {
                    existing_tx.chargeback()?;
                    let client = clients.get_client(tx.client_id);
                    client.freeze();
                    client.chargeback(existing_tx.amount.unwrap())?;
                }
                TxType::Withdrawal | TxType::Deposit => return Err(Box::new(DuplicateTransaction)),
            }
        } else {
            match tx.tx_type {
                TxType::Deposit => {
                    clients
                        .get_client(tx.client_id)
                        .deposit(tx.amount.unwrap())?;
                    ledger.insert_transaction(&tx);
                }
                TxType::Withdrawal => {
                    clients
                        .get_client(tx.client_id)
                        .withdrawal(tx.amount.unwrap())?;
                    ledger.insert_transaction(&tx)
                }
                TxType::Chargeback | TxType::Resolve | TxType::Dispute => {
                    return Err(Box::new(NonExistingTransaction))
                }
            }
        }
        Ok(())
    }
}

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
