use crate::{common::*, transaction::*, transaction_engine::TransactionEngine};
use anyhow::{bail, Result};
use client::Client;
use serde::Deserialize;
use std::{env, io};

mod client;
mod common;
mod message;
mod transaction;
mod transaction_engine;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("ERROR: Expected CSV file as input parameter.");
    }

    let transactions = load_transactions(&args[1])?;

    // transactions.iter().for_each(|tx| eprintln!("{:?}", tx));

    let mut messages = vec![];
    let mut te = TransactionEngine::default();
    let mut res = Ok(());
    for tx in transactions {
        match te.process(tx, &mut messages) {
            Err(err) => {
                res = Err(err);
                break;
            }
            _ => {}
        }
    }

    messages.iter().for_each(|m| eprintln!("{:?}", m));

    print_clients(&te.clients())?;

    res
}

fn load_transactions(path: &str) -> Result<Vec<Tx>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)?;
    let mut transactions = vec![];

    for record in reader.deserialize() {
        let input_tx: InputTx = record?;

        input_tx.validate()?;

        transactions.push(input_tx.into());
    }

    Ok(transactions)
}

fn print_clients(clients: &[&Client]) -> Result<()> {
    let mut writer = csv::Writer::from_writer(io::stdout());

    for client in clients {
        writer.serialize(client)?;
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct InputTx {
    #[serde(rename = "type")]
    pub ty: TxType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<Value>,
}

impl From<InputTx> for Tx {
    fn from(tx: InputTx) -> Self {
        Self {
            ty: tx.ty,
            client_id: tx.client,
            tx_id: tx.tx,
            amount: tx.amount.unwrap_or_default(),
            state: TxState::Active,
        }
    }
}

impl InputTx {
    fn validate(&self) -> Result<()> {
        match self.ty {
            TxType::Deposit | TxType::Withdrawal => match self.amount {
                Some(amount) if amount <= 0.0 => {
                    bail!(
                        "ERROR: {:?} transaction {} for client {} contains negative amount.",
                        self.ty,
                        self.tx,
                        self.client
                    );
                }
                None => {
                    bail!(
                        "ERROR: {:?} transaction {} for client {} contains no amount.",
                        self.ty,
                        self.tx,
                        self.client
                    );
                }
                _ => (),
            },
            _ => {
                if self.amount.is_some() {
                    eprintln!(
                        "WARNING: {:?} transaction {} for client {} should not contain amount.",
                        self.ty, self.tx, self.client
                    );
                }
            }
        }
        Ok(())
    }
}
