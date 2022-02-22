use crate::core::ClientId;
use crate::utils::build_custom_error;
use serde::Deserialize;
use std::{collections::HashMap, error::Error, fmt};

type TransactionId = u32;

#[derive(Default, Debug)]
pub(crate) struct Ledger(HashMap<TransactionId, Tx>);

impl Ledger {
    pub(crate) fn get_transaction(&mut self, tx: &Tx) -> Option<&mut Tx> {
        self.0.get_mut(&tx.id)
    }
    pub(crate) fn insert_transaction(&mut self, tx: &Tx) {
        self.0.insert(tx.id, tx.clone());
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TxType {
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

#[derive(Deserialize, Debug, Clone)]
pub struct Tx {
    #[serde(alias = "type")]
    pub(crate) tx_type: TxType,
    #[serde(alias = "client")]
    pub(crate) client_id: ClientId,
    #[serde(alias = "tx")]
    id: TransactionId,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub(crate) amount: Option<Decimal>,
    tx_state: Option<TxState>,
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

impl Tx {
    pub(crate) fn dispute(&mut self) -> Result<(), UndisputableTransaction> {
        if self.tx_type != TxType::Withdrawal
            && (self.tx_state == Some(TxState::Resolved) || self.tx_state.is_none())
        {
            self.tx_state = Some(TxState::Disputed);
            Ok(())
        } else {
            Err(UndisputableTransaction)
        }
    }
    pub(crate) fn chargeback(&mut self) -> Result<(), IrreversableTransaction> {
        if let Some(TxState::Disputed) = self.tx_state {
            self.tx_state = Some(TxState::Chargebacked);
            Ok(())
        } else {
            Err(IrreversableTransaction)
        }
    }
    pub(crate) fn resolve(&mut self) -> Result<(), UnresolvableTransaction> {
        if Some(TxState::Disputed) == self.tx_state {
            self.tx_state = Some(TxState::Resolved);
            Ok(())
        } else {
            Err(UnresolvableTransaction)
        }
    }
}

impl fmt::Display for Tx {
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
