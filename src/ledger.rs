#![allow(dead_code)]

use crate::core::ClientId;
use crate::utils::build_custom_error;
use rust_decimal::prelude::*;
use serde::{de, Deserialize};
use std::{any::Any, collections::HashMap, error::Error, fmt};

type TransactionId = u32;
type BoxedTransaction = Box<dyn Any + 'static>;

#[derive(Default, Debug)]
pub(crate) struct Ledger(HashMap<TransactionId, BoxedTransaction>);

impl Ledger {
    pub(crate) fn get_transaction<S>(&mut self, tx: &Tx<S>) -> Option<&mut BoxedTransaction> {
        self.0.get_mut(&tx.id)
    }
    pub(crate) fn insert_transaction<S: 'static>(&mut self, tx: Tx<S>) {
        self.0.insert(tx.id, Box::new(tx));
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub(crate) struct DefaultState;
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub(crate) struct DisputedState;
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub(crate) struct ChargebackedState;

impl Tx<DefaultState> {
    pub(crate) fn new(tx_type: TxType, amount: &str) -> Self {
        Tx::<DefaultState> {
            amount: Decimal::from_str_exact(amount).ok(),
            client_id: 8,
            id: 8,
            tx_type: TxType::Chargeback,
            tx_state: DefaultState,
        }
    }
    pub(crate) fn dispute(self) -> Result<Tx<DisputedState>, UndisputableTransaction> {
        if self.tx_type != TxType::Withdrawal {
            Ok(Tx::<DisputedState> {
                tx_state: DisputedState,
                ..self
            })
        } else {
            Err(UndisputableTransaction)
        }
    }
}
impl Tx<DisputedState> {
    pub(crate) fn resovle(self) -> Tx<DefaultState> {
        Tx::<DefaultState> {
            tx_state: DefaultState,
            ..self
        }
    }
    pub(crate) fn chargeback(self) -> Tx<ChargebackedState> {
        Tx::<ChargebackedState> {
            tx_state: ChargebackedState,
            ..self
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TxType {
    Chargeback,
    Resolve,
    Dispute,
    Withdrawal,
    Deposit,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
enum TxState {
    Disputed,
    Resolved,
    Chargebacked,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
pub struct Tx<State> {
    #[serde(alias = "type")]
    pub(crate) tx_type: TxType,
    #[serde(alias = "client")]
    pub(crate) client_id: ClientId,
    #[serde(alias = "tx")]
    pub(crate) id: TransactionId,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub(crate) amount: Option<Decimal>,
    #[serde(skip)]
    pub(crate) tx_state: State,
}

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    match Decimal::from_str_exact(s) {
        Ok(v) => Ok(Some(v)),
        Err(_) => Ok(None),
    }
}

impl<State: std::fmt::Debug> fmt::Display for Tx<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "type:{:?},client:{},tx:{},amount:{:?},state:{:?}",
            self.tx_type, self.client_id, self.id, self.amount, self.tx_state
        )
    }
}
build_custom_error!(
    IrreversableTransaction,
    "Attempted to chargeback a transaction which is currently not disputed."
);
build_custom_error!(
    UnresolvableTransaction,
    "Attempted to resolve a transaction which is currently not disputed."
);
build_custom_error!(
    UndisputableTransaction,
    "Withdrawal transaction cannot be disputed."
);

#[cfg(test)]
mod tests {

    use super::*;

    // im// pl<State> Tx<State> {
    //     fn new(tx_type: TxType, amount: &str, tx_state: State) -> Tx<State> {
    //         Tx::<State> {
    //             tx_type,
    //             client_id: 1,
    //             id: 1,
    //             amount: Decimal::from_str_exact(amount).ok(),
    //             tx_state,
    //         }
    //     }
    // }

    #[test]
    fn successful_tx_ledger_crud() {
        let mut l = Ledger::default();
        let t = Tx::<DefaultState>::new(TxType::Deposit, "100.02");

        l.insert_transaction(t);
        assert!(l.0.contains_key(&t.id));
        // let crud_t = l.get_transaction(&t).unwrap().to_owned();
        // assert!(crud_t == t);
    }

    #[test]
    fn successful_resolve_after_dispute() -> Result<(), Box<dyn Error>> {
        let mut t = Tx::new(TxType::Deposit, "100.02");
        assert!(t.tx_state == DefaultState);
        let op = t.dispute();
        assert!(op.is_ok());
        let x = op?;
        assert!(x.tx_state == DisputedState);
        let op = x.resovle();
        // assert!(op.is_ok());
        assert!(t.tx_state == DefaultState);
        Ok(())
    }

    // #[test]
    // fn successful_chargeback_after_dispute() {
    //     let mut t = Tx::new(TxType::Deposit, "100.02", None);
    //     assert!(t.tx_state == None);
    //     let op = t.dispute();
    //     assert!(op.is_ok());
    //     assert!(t.tx_state == Some(TxState::Disputed));
    //     let op = t.chargeback();
    //     assert!(op.is_ok());
    //     assert!(t.tx_state == Some(TxState::Chargebacked));
    // }

    // #[should_panic]
    // #[test]
    // fn failed_tx_ledger_crud() {
    //     let mut l = Ledger::default();
    //     let t = Tx::new(TxType::Deposit, "100.02", None);
    //     l.get_transaction(&t).unwrap();
    // }

    // #[test]
    // fn failed_chargeback_after_resolve() {
    //     let mut t = Tx::new(TxType::Deposit, "100.02", None);
    //     assert!(t.tx_state == None);
    //     let op = t.dispute();
    //     assert!(op.is_ok());
    //     assert!(t.tx_state == Some(TxState::Disputed));
    //     let op = t.chargeback();
    //     assert!(op.is_ok());
    //     assert!(t.tx_state == Some(TxState::Chargebacked));
    // }

    // #[should_panic]
    // #[test]
    // fn failed_double_dispute() {
    //     let mut t = Tx::new(TxType::Deposit, "100.02", None);
    //     assert!(t.dispute().is_ok());
    //     t.dispute().unwrap();
    // }

    // #[should_panic]
    // #[test]
    // fn failed_double_resolve() {
    //     let mut t = Tx::new(TxType::Deposit, "100.02", None);
    //     assert!(t.resolve().is_ok());
    //     t.resolve().unwrap();
    // }

    // #[should_panic]
    // #[test]
    // fn failed_double_chargeback() {
    //     let mut t = Tx::new(TxType::Deposit, "100.02", None);
    //     assert!(t.chargeback().is_ok());
    //     t.chargeback().unwrap();
    // }
}
