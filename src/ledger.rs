//! Transactions and their storage

use crate::core::ClientId;
use crate::utils::build_custom_error;
use rust_decimal::prelude::*;
use serde::{de, Deserialize};
use std::{any::Any, collections::HashMap, error::Error, fmt};
pub(crate) use transaction::*;
use typestate::typestate;

type TransactionId = u32;
type BoxedTransaction = Box<dyn Any + 'static>;

/// Storage for transactions. Holds mapping between transaction ID and the transaction itself
#[derive(Default, Debug)]
pub(crate) struct Ledger(HashMap<TransactionId, BoxedTransaction>);

impl Ledger {
    pub(crate) fn get_transaction_by_id(
        &mut self,
        tx_id: TransactionId,
    ) -> Option<&mut BoxedTransaction> {
        self.0.get_mut(&tx_id)
    }
    pub(crate) fn insert_transaction<S: 'static + TxState>(&mut self, tx: Tx<S>) {
        self.0.insert(tx.id, Box::new(tx));
    }
}

/// Transaction types
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

    /// A single transaction.
    /// This struct is used both for ser/deserializing by `sered`, and
    /// by `typestate` crate to generate `<TxState>` for this struct.
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

    /// Default transaction state assigned to transactions which
    /// have been parsed. If trasaction is of type Withdrawal or Dispute,
    /// and has never been *disputed*, it will remain in Defualt state
    #[state]
    #[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
    pub(crate) struct Default;
    /// Transaction state assigned to transactions which has been *disputed*
    #[state]
    #[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
    pub(crate) struct Disputed;
    /// Transaction state assigned to transactions after disputed transaction
    /// has been *chargebacked* (returned)
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
    fn default() -> Tx<transaction::Default> {
        Tx::<transaction::Default> {
            amount: Some(Decimal::ONE),
            state: transaction::Default,
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
            state: transaction::Default,
            client_id: 8,
            id: 9,
            tx_type: TxType::Chargeback,
        }
    }
    fn dispute(self) -> Tx<Disputed> {
        Tx::<Disputed> {
            // repetition easliy avoidable with `..self`
            // but will have to wait once this is stable
            // https://github.com/rust-lang/rust/issues/86555
            state: Disputed,
            amount: self.amount,
            client_id: self.client_id,
            id: self.id,
            tx_type: self.tx_type,
        }
    }
}

impl DisputedState for Tx<Disputed> {
    fn resolve(self) -> Tx<transaction::Default> {
        Tx::<Default> {
            state: Default,
            amount: self.amount,
            client_id: self.client_id,
            id: self.id,
            tx_type: self.tx_type,
        }
    }
    fn chargeback(self) -> Tx<Chargebacked> {
        Tx::<Chargebacked> {
            state: Chargebacked,
            amount: self.amount,
            client_id: self.client_id,
            id: self.id,
            tx_type: self.tx_type,
        }
    }
}

build_custom_error!(
    IrreversableTransaction,
    "ERROR: Attempted to chargeback a transaction which is currently not disputed."
);
build_custom_error!(
    UnresolvableTransaction,
    "ERROR: Attempted to resolve a transaction which is currently not disputed."
);
build_custom_error!(
    UndisputableTransaction,
    "ERROR: Withdrawal transaction cannot be disputed."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_tx_ledger_crud() {
        let mut l = Ledger::default();
        let t = Tx::<Default>::create();
        let t = t.dispute();
        l.insert_transaction(t);
        assert!(l.0.contains_key(&t.id));
        let crud_tx = l.get_transaction_by_id(t.id).unwrap();
        if let Some(crud_tx) = crud_tx.downcast_ref::<Tx<Disputed>>() {
            assert!(crud_tx.to_owned() == t);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn insert_to_ledger_transactions_with_different_types() {
        let mut l = Ledger::default();
        let mut t = Tx::<Default>::create();
        t.id = 11; // avoid key collision in HashMap
        let t = t.dispute();
        l.insert_transaction(t);
        let t = Tx::<Default>::create();
        let t = t.dispute();
        let t = t.chargeback();
        l.insert_transaction(t);
        assert_eq!(l.0.values().len(), 2);
    }

    #[test]
    fn successful_resolve_after_dispute() -> Result<(), Box<dyn Error>> {
        let tx = Tx::<Default>::create();
        assert!(tx.state == Default);
        let tx = tx.dispute();
        assert!(tx.state == Disputed);
        let tx = tx.resolve();
        assert!(tx.state == Default);
        Ok(())
    }

    #[test]
    fn successful_chargeback_after_dispute() {
        let tx = Tx::<Default>::create();
        assert!(tx.state == Default);
        let tx = tx.dispute();
        assert!(tx.state == Disputed);
        let tx = tx.chargeback();
        assert!(tx.state == Chargebacked);
    }

    #[should_panic]
    #[test]
    fn failed_tx_ledger_crud() {
        let mut l = Ledger::default();
        l.get_transaction_by_id(888).unwrap();
    }
}
