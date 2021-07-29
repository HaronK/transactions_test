use crate::{message::*, transaction::*};
use anyhow::{bail, Result};
use client::Client;
use std::{collections::HashMap, env, io};

mod client;
mod common;
mod message;
mod transaction;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("ERROR: Expected CSV file as input parameter.");
    }

    let transactions = load_transactions(&args[1])?;

    // transactions.iter().for_each(|tx| eprintln!("{:?}", tx));

    let mut messages = vec![];
    let clients_res = process(&transactions, &mut messages);

    messages.iter().for_each(|m| eprintln!("{:?}", m));

    match clients_res {
        Ok(clients) => {
            // println!("Clients: {:#?}", clients);

            print_clients(clients)
        }
        Err(err) => Err(err),
    }
}

fn load_transactions(path: &str) -> Result<Vec<Tx>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)?;
    let mut transactions = vec![];

    for record in reader.deserialize() {
        let input_tx: InputTx = record?;

        validate(&input_tx)?;

        transactions.push(input_tx.into());
    }

    Ok(transactions)
}

fn print_clients(clients: Vec<Client>) -> Result<()> {
    let mut writer = csv::Writer::from_writer(io::stdout());

    for client in clients {
        writer.serialize(client)?;
    }

    Ok(())
}

fn validate(tx: &InputTx) -> Result<()> {
    match tx.ty {
        TxType::Deposit | TxType::Withdrawal => match tx.amount {
            Some(amount) if amount <= 0.0 => {
                bail!(
                    "ERROR: {:?} transaction {} for client {} contains negative amount.",
                    tx.ty,
                    tx.tx,
                    tx.client
                );
            }
            None => {
                bail!(
                    "ERROR: {:?} transaction {} for client {} contains no amount.",
                    tx.ty,
                    tx.tx,
                    tx.client
                );
            }
            _ => (),
        },
        _ => {
            if tx.amount.is_some() {
                eprintln!(
                    "WARNING: {:?} transaction {} for client {} should not contain amount.",
                    tx.ty, tx.tx, tx.client
                );
            }
        }
    }
    Ok(())
}

fn process(transactions: &[Tx], messages: &mut Vec<Message>) -> Result<Vec<Client>> {
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
