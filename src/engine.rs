use crate::core::{Accounts, Ledger, Tx, TxType};
use crate::utils::build_custom_error;
use std::{error::Error, fmt};

#[derive(Default)]
pub(crate) struct PaymentEngine;

impl PaymentEngine {
    pub(crate) fn process_transaction(
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

build_custom_error!(
    DuplicateTransaction,
    "Deposit or Withdrawal transaction with the same ID already exists in the ledger."
);
build_custom_error!(
    NonExistingTransaction,
    "Attempted to postprocess a non existent transaction."
);

#[cfg(test)]
mod tests {
    #[should_panic]
    #[test]
    fn test() {
        todo!()
    }
}
