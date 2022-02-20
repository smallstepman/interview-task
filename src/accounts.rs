use crate::utils::build_custom_error;
use csv::Writer;
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
        for client in self.0.values() {
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
    available: f32,
    held: f32,
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
    pub(crate) fn freeze(&mut self) {
        self.locked = true;
    }
    pub(crate) fn deposit(&mut self, amount: f32) -> ClientActionResult {
        self.is_unlocked()?;
        self.available += amount;
        Ok(())
    }
    pub(crate) fn withdrawal(&mut self, amount: f32) -> ClientActionResult {
        self.is_unlocked()?;
        self.has_enough_funds(amount)?;
        self.available -= amount;
        Ok(())
    }
    pub(crate) fn hold(&mut self, amount: f32) -> ClientActionResult {
        self.is_unlocked()?;
        self.has_enough_funds(amount)?;
        self.available -= amount;
        self.held += amount;
        Ok(())
    }
    pub(crate) fn resolve(&mut self, amount: f32) -> ClientActionResult {
        self.is_unlocked()?;
        self.held -= amount;
        self.available += amount;
        Ok(())
    }
    pub(crate) fn chargeback(&mut self, amount: f32) -> ClientActionResult {
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

impl Serialize for Client {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Client", 5)?;
        s.serialize_field("cilent", &self.id)?;
        s.serialize_field("available", &self.available)?;
        s.serialize_field("held", &self.held)?;
        s.serialize_field("total", &(&self.available + &self.held))?;
        s.serialize_field("locked", &self.locked)?;
        s.end()
    }
}

build_custom_error!(NotEnoughFunds, "Not enough funds in client's account.");
build_custom_error!(AccountLocked, "Unable to modify locked account.");
