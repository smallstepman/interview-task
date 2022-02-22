use crate::utils::build_custom_error;
use csv::Writer;
use rust_decimal::Decimal;
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use std::collections::HashMap;
use std::{error::Error, fmt};

pub(crate) type ClientId = u16;
type ClientActionResult = Result<(), Box<dyn Error>>;

#[derive(Default, Debug)]
pub(crate) struct Accounts(HashMap<ClientId, Client>);

impl Accounts {
    pub(crate) fn get_client(&mut self, id: ClientId) -> &mut Client {
        self.0.entry(id).or_insert(Client::new(id))
    }

    pub fn report_accounts_balances(&self) -> Result<String, Box<dyn Error>> {
        let mut wtr = Writer::from_writer(vec![]);
        // Hashmap's hashing algorithm is randomly seeded.
        // Here, we're collecting HashMap values into Vec, to then sort the clients
        // based on their ID, because this ensures deterministic output of this function
        // which in turn makes testing much easier as there is no need to account for
        // all different output combinations of the same data. This affects performance,
        // but given the purposes or this task, I'm happy to accept it.
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
pub(crate) struct Client {
    id: ClientId,
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
    pub(crate) fn deposit(&mut self, amount: Decimal) -> ClientActionResult {
        self.is_unlocked()?;
        self.available += amount;
        Ok(())
    }
    pub(crate) fn withdrawal(&mut self, amount: Decimal) -> ClientActionResult {
        self.is_unlocked()?;
        self.has_enough_funds(amount)?;
        self.available -= amount;
        Ok(())
    }
    pub(crate) fn hold(&mut self, amount: Decimal) -> ClientActionResult {
        self.is_unlocked()?;
        self.has_enough_funds(amount)?;
        self.available -= amount;
        self.held += amount;
        Ok(())
    }
    pub(crate) fn resolve(&mut self, amount: Decimal) -> ClientActionResult {
        self.is_unlocked()?;
        self.held -= amount;
        self.available += amount;
        Ok(())
    }
    pub(crate) fn chargeback(&mut self, amount: Decimal) -> ClientActionResult {
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
}

impl Serialize for Client {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let round = |d: Decimal| {
            let mut normalized = d.normalize();
            if normalized.scale() == 0 {
                normalized.rescale(1);
            } else if normalized.scale() > 4 {
                normalized = normalized.round_dp(4);
            }
            normalized
        };
        let mut s = serializer.serialize_struct("Client", 5)?;
        s.serialize_field("client", &self.id)?;
        s.serialize_field("available", &round(self.available))?;
        s.serialize_field("held", &round(self.held))?;
        s.serialize_field("total", &round(self.calculate_total()))?;
        s.serialize_field("locked", &self.locked)?;
        s.end()
    }
}

build_custom_error!(NotEnoughFunds, "Not enough funds in client's account.");
build_custom_error!(AccountLocked, "Unable to modify locked account.");
