use crate::core::{
    transaction::{self, Chargebacked, DefaultState, Disputed, DisputedState, Tx, TxState},
    Accounts, Ledger, TxType,
};
use crate::utils::build_custom_error;
use std::{any::Any, error::Error, fmt};

#[derive(Default)]
pub(crate) struct PaymentEngine;

impl PaymentEngine {
    pub(crate) fn process_transaction<
        S: Any + TxState, // , OtherState: Clone + Copy
    >(
        &mut self,
        incoming_tx: Tx<S>,
        ledger: &mut Ledger,
        clients: &mut Accounts,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(referenced_tx) = ledger.get_transaction(&incoming_tx) {
            if incoming_tx.tx_type == TxType::Deposit || incoming_tx.tx_type == TxType::Withdrawal {
                return Err(Box::new(DuplicateTransaction));
            }
            if let Some(referenced_tx) = referenced_tx.downcast_mut::<Tx<transaction::Default>>() {
                if incoming_tx.tx_type == TxType::Dispute {
                    let client = clients.get_client(referenced_tx.client_id);
                    if client.id == incoming_tx.client_id {
                        let _ = referenced_tx.dispute();
                        client.hold(referenced_tx.amount.unwrap())?;
                    }
                } else {
                    return Err(Box::new(InvalidRequestError));
                }
            } else if let Some(referenced_tx) = referenced_tx.downcast_mut::<Tx<Disputed>>() {
                let client = clients.get_client(referenced_tx.client_id);
                if client.id != incoming_tx.client_id {
                    return Err(Box::new(InvalidRequestError));
                }
                match incoming_tx.tx_type {
                    TxType::Resolve => {
                        let _ = referenced_tx.resolve();
                        client.resolve(referenced_tx.amount.unwrap())?;
                    }
                    TxType::Chargeback => {
                        let _ = referenced_tx.chargeback();
                        client.freeze();
                        client.chargeback(referenced_tx.amount.unwrap())?;
                    }
                    _ => return Err(Box::new(InvalidRequestError)),
                }
            } else if let Some(_referenced_tx) = referenced_tx.downcast_mut::<Tx<Chargebacked>>() {
                return Err(Box::new(ChargedbackTransacionError));
            }
        } else if incoming_tx.tx_type == TxType::Chargeback
            || incoming_tx.tx_type == TxType::Resolve
            || incoming_tx.tx_type == TxType::Dispute
        {
            return Err(Box::new(NonExistingTransaction));
        } else {
            let client = clients.get_client(incoming_tx.client_id);
            if incoming_tx.tx_type == TxType::Deposit {
                client.deposit(incoming_tx.amount.unwrap())?;
            } else if incoming_tx.tx_type == TxType::Withdrawal {
                client.withdrawal(incoming_tx.amount.unwrap())?;
            }
            ledger.insert_transaction(incoming_tx);
        }
        Ok(())
    }
}

build_custom_error!(
    DuplicateTransaction,
    "Deposit or Withdrawal transaction with the same ID already exists in the ledger."
);
build_custom_error!(
    NonExistingTransaction,
    "Attempted to postprocess a non existent transaction."
);
build_custom_error!(
    ChargedbackTransacionError,
    "Transaction has been chargedback - no further action is possible."
);
build_custom_error!(
    InvalidRequestError,
    "Transaction has been chargedback - no further action is possible."
);

#[cfg(test)]
mod tests {
    #[should_panic]
    #[test]
    fn test() {
        todo!()
    }
}
