use crate::{client::Client, message::Message, transaction::Tx};
use anyhow::Result;
use std::collections::HashMap;

pub fn process(transactions: &[Tx], messages: &mut Vec<Message>) -> Result<Vec<Client>> {
    let mut clients = HashMap::new();

    for tx in transactions {
        if !clients.contains_key(&tx.client_id) {
            clients.insert(tx.client_id, Client::new(tx.client_id));
        }

        let client = clients.get_mut(&tx.client_id).unwrap();

        client.process(tx, messages);
    }

    Ok(clients.drain().map(|(_, v)| v).collect())
}
