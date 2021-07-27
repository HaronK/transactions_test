use crate::{process::process, transaction::*};
use anyhow::{bail, Result};
use data::TxType;
use std::{env, io};

mod client;
mod data;
mod process;
mod transaction;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("ERROR: Expected CSV file as input parameter.");
    }

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(&args[1])?;
    let mut transactions = vec![];

    for record in reader.deserialize() {
        let input_tx: InputTx = record?;

        validate(&input_tx)?;

        transactions.push(input_tx.into());
    }

    // transactions.iter().for_each(|t| eprintln!("{:?}", t));

    let clients = process(&transactions)?;

    // println!("Clients: {:#?}", clients);

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
