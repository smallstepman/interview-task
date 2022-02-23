//! Client's account and their storage

use crate::utils::build_custom_error;
use csv::Writer;
use rust_decimal::Decimal;
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use std::{collections::HashMap, error::Error, fmt};

pub(crate) type ClientId = u16;
type ClientResult = Result<(), Box<dyn Error>>;

/// Storage for client's accounts. Maps between client ID and their account.
#[derive(Default, Debug)]
pub(crate) struct Accounts(HashMap<ClientId, Client>);

impl Accounts {
    pub(crate) fn get_client(&mut self, id: ClientId) -> &mut Client {
        self.0.entry(id).or_insert(Client::new(id))
    }

    /// Build csv report file, from all of the accounts
    pub(crate) fn report_accounts_balances(&self) -> Result<String, Box<dyn Error>> {
        let mut wtr = Writer::from_writer(vec![]);
        // Hashmap's hashing algorithm is randomly seeded.
        // Here, we're collecting HashMap values into Vec, to then sort the clients
        // based on their ID, because this ensures deterministic output of this function
        // which in turn makes integration testing much easier as there is no need to
        // account for all different output combinations of the same data.
        // This affects performance, but given the purposes or this task, I'm happy to accept it.
        let mut clients = self.0.values().collect::<Vec<_>>();
        clients.sort_by_key(|c| c.id);
        for client in clients {
            wtr.serialize(client)?;
        }
        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }
}

#[derive(Debug, Clone)]
/// A single client's account
pub(crate) struct Client {
    pub(crate) id: ClientId,
    locked: bool,
    available: Decimal,
    held: Decimal,
}

impl Client {
    fn new(id: ClientId) -> Self {
        Client {
            id,
            locked: false,
            available: Decimal::new(0, 4),
            held: Decimal::new(0, 4),
        }
    }
    pub(crate) fn freeze(&mut self) {
        self.locked = true;
    }
    pub(crate) fn deposit(&mut self, amount: Decimal) -> ClientResult {
        self.is_unlocked()?;
        self.available += amount;
        Ok(())
    }
    pub(crate) fn withdrawal(&mut self, amount: Decimal) -> ClientResult {
        self.is_unlocked()?;
        self.has_enough_funds(amount)?;
        self.available -= amount;
        Ok(())
    }
    pub(crate) fn hold(&mut self, amount: Decimal) -> ClientResult {
        self.is_unlocked()?;
        self.has_enough_funds(amount)?;
        self.available -= amount;
        self.held += amount;
        Ok(())
    }
    pub(crate) fn resolve(&mut self, amount: Decimal) -> ClientResult {
        self.is_unlocked()?;
        self.held -= amount;
        self.available += amount;
        Ok(())
    }
    pub(crate) fn chargeback(&mut self, amount: Decimal) -> ClientResult {
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
    fn has_enough_funds(&mut self, amount: Decimal) -> Result<(), NotEnoughFunds> {
        if self.available >= amount {
            Ok(())
        } else {
            Err(NotEnoughFunds)
        }
    }
    fn calculate_total(&self) -> Decimal {
        self.held + self.available
    }
    /// Round `rust_decimal` structs according to specification.
    /// `0.0000001` -> `0.0`, `0` -> `0.0`, `0.010` -> `0.01`
    fn round(d: Decimal) -> Decimal {
        let mut normalized = d.normalize();
        if normalized.scale() == 0 {
            normalized.rescale(1);
        } else if normalized.scale() > 4 {
            normalized = normalized.round_dp(4);
        }
        normalized
    }
}

impl Serialize for Client {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Client", 5)?;
        s.serialize_field("client", &self.id)?;
        s.serialize_field("available", &Self::round(self.available))?;
        s.serialize_field("held", &Self::round(self.held))?;
        s.serialize_field("total", &Self::round(self.calculate_total()))?;
        s.serialize_field("locked", &self.locked)?;
        s.end()
    }
}

build_custom_error!(
    NotEnoughFunds,
    "ERROR: Not enough funds in client's account."
);
build_custom_error!(AccountLocked, "ERROR: Unable to modify locked account.");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decline_overdraft_attempt() {
        let mut c = Client::new(123);
        c.available = Decimal::ONE;
        let op = c.withdrawal(Decimal::new(100, 0));
        assert!(op.is_err());
    }

    #[test]
    fn decline_withdrawal_from_frozen_account() {
        let mut c = Client::new(123);
        c.available = Decimal::ONE;
        c.locked = true;
        let op = c.withdrawal(Decimal::new(1, 1));
        assert!(op.is_err());
    }

    #[test]
    fn correct_decimal_formatting() {
        let mut c = Client::new(123);
        c.available = Decimal::new(12322344, 5);
        assert_eq!(Client::round(c.available).to_string(), "123.2234");
        c.available = Decimal::new(12322344, 0);
        assert_eq!(Client::round(c.available).to_string(), "12322344.0");
        c.available = Decimal::new(1440000000, 7);
        assert_eq!(Client::round(c.available).to_string(), "144.0");
        c.available = Decimal::new(1440200000, 7);
        assert_eq!(Client::round(c.available).to_string(), "144.02");
        c.available = Decimal::new(1440020000, 7);
        assert_eq!(Client::round(c.available).to_string(), "144.002");
        c.available = Decimal::new(1440002001, 7);
        assert_eq!(Client::round(c.available).to_string(), "144.0002");
    }

    #[test]
    fn correct_floating_point_operations() {
        let mut c = Client::new(123);
        c.deposit(Decimal::new(123223412341243344, 14)).ok();
        c.deposit(Decimal::new(123223777700000007, 16)).ok();
        assert_eq!(Client::round(c.available).to_string(), "1244.5565");
    }

    #[test]
    fn freeze_account() {
        let mut c = Client::new(123);
        c.freeze();
        assert!(c.locked);
    }
}
