use crate::core::ClientId;
use crate::utils::build_custom_error;
use rust_decimal::prelude::*;
use serde::{de, Deserialize};
use std::{any::Any, collections::HashMap, error::Error, fmt};

pub(crate) use transaction::*;
use typestate::typestate;

type TransactionId = u32;
type BoxedTransaction = Box<dyn Any + 'static>;

#[derive(Default, Debug)]
pub(crate) struct Ledger(HashMap<TransactionId, BoxedTransaction>);

impl Ledger {
    pub(crate) fn get_transaction<S: TxState>(
        &mut self,
        tx: &Tx<S>,
    ) -> Option<&mut BoxedTransaction> {
        self.0.get_mut(&tx.id)
    }
    pub(crate) fn insert_transaction<S: 'static + TxState>(&mut self, tx: Tx<S>) {
        self.0.insert(tx.id, Box::new(tx));
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

#[typestate]
pub(crate) mod transaction {
    use super::{de, fmt, ClientId, Decimal, Deserialize, TransactionId, TxType};

    #[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
    #[serde(default, deny_unknown_fields)]
    #[automaton]
    pub struct Tx {
        #[serde(alias = "type")]
        pub(crate) tx_type: TxType,
        #[serde(alias = "client")]
        pub(crate) client_id: ClientId,
        #[serde(alias = "tx")]
        pub(crate) id: TransactionId,
        #[serde(deserialize_with = "deserialize_decimal")]
        pub(crate) amount: Option<Decimal>,
    }

    #[state]
    #[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
    pub(crate) struct Default;
    #[state]
    #[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
    pub(crate) struct Disputed;
    #[state]
    #[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
    pub(crate) struct Chargebacked;

    pub(crate) trait Default {
        fn create() -> Default;
        fn dispute(self) -> Disputed;
    }
    pub(crate) trait Disputed {
        fn resolve(self) -> Default;
        fn chargeback(self) -> Chargebacked;
    }
    pub(crate) trait Chargebacked {
        fn archive(self);
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

    impl<State: std::fmt::Debug + TxState> fmt::Display for Tx<State> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "type:{:?},client:{},tx:{},amount:{:?},state:{:?}",
                self.tx_type, self.client_id, self.id, self.amount, self.state
            )
        }
    }
}

impl std::default::Default for Tx<Default> {
    fn default() -> Tx<Default> {
        Tx::<transaction::Default> {
            amount: Some(Decimal::ONE),
            state: Default,
            client_id: 0,
            id: 0,
            tx_type: TxType::Chargeback,
        }
    }
}

impl DefaultState for Tx<transaction::Default> {
    fn create() -> Tx<transaction::Default> {
        Tx::<transaction::Default> {
            amount: Some(Decimal::ONE),
            state: Default,
            client_id: 8,
            id: 9,
            tx_type: TxType::Chargeback,
        }
    }
    fn dispute(self) -> Tx<Disputed> {
        Tx::<Disputed> {
            state: Disputed,
            ..self
        }
    }
}

impl DisputedState for Tx<Disputed> {
    fn resolve(self) -> Tx<transaction::Default> {
        Tx::<Default> {
            state: Default,
            ..self
        }
    }
    fn chargeback(self) -> Tx<Chargebacked> {
        Tx::<Chargebacked> {
            state: Chargebacked,
            ..self
        }
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

    // use super::*;

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

    // #[test]
    // fn successful_tx_ledger_crud() {
    //     let mut l = Ledger::default();
    //     let t = Tx::<DefaultState>::new(TxType::Deposit, "100.02");

    //     l.insert_transaction(t);
    //     assert!(l.0.contains_key(&t.id));
    //     // let crud_t = l.get_transaction(&t).unwrap().to_owned();
    //     // assert!(crud_t == t);
    // }

    // #[test]
    // fn successful_resolve_after_dispute() -> Result<(), Box<dyn Error>> {
    //     let mut t = Tx::new(TxType::Deposit, "100.02");
    //     assert!(t.tx_state == DefaultState);
    //     let op = t.dispute();
    //     assert!(op.is_ok());
    //     let x = op?;
    //     assert!(x.tx_state == DisputedState);
    //     let op = x.resovle();
    //     // assert!(op.is_ok());
    //     assert!(t.tx_state == DefaultState);
    //     Ok(())
    // }

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
