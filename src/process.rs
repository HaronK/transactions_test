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

#[cfg(test)]
mod tests {
    use crate::{message::Message, transaction::*};
    use anyhow::Result;
    use helper::*;

    #[test]
    fn test_empty() -> Result<()> {
        test_process(&[], &[], &[])
    }

    #[test]
    fn test_single_transaction() -> Result<()> {
        test_process(
            &[tx_deposit(1, 1, 5.0)],
            &[client(1, 5.0, 0.0, 5.0, false)],
            &[],
        )
    }

    #[test]
    fn test_transaction_same_id_fail() -> Result<()> {
        test_process(
            &[tx_deposit(1, 1, 5.0), tx_deposit(1, 1, 2.0)],
            &[client(1, 5.0, 0.0, 5.0, false)],
            &[Message::TransactionExist(1, 1, TxType::Deposit)],
        )
    }

    #[test]
    fn test_withdrawal_full() -> Result<()> {
        test_process(
            &[tx_deposit(1, 1, 5.0), tx_withdrawal(1, 2, 5.0)],
            &[client(1, 0.0, 0.0, 0.0, false)],
            &[],
        )
    }

    #[test]
    fn test_withdrawal_partial() -> Result<()> {
        test_process(
            &[tx_deposit(1, 1, 5.0), tx_withdrawal(1, 2, 2.0)],
            &[client(1, 3.0, 0.0, 3.0, false)],
            &[],
        )
    }

    #[test]
    fn test_withdrawal_exeeding() -> Result<()> {
        test_process(
            &[tx_deposit(1, 1, 5.0), tx_withdrawal(1, 2, 7.0)],
            &[client(1, 5.0, 0.0, 5.0, false)],
            &[Message::NotEnoughFunds(1, 2, TxType::Withdrawal)],
        )
    }

    #[test]
    fn test_deposit_dispute() -> Result<()> {
        test_process(
            &[tx_deposit(1, 1, 5.0), tx_dispute(1, 1)],
            &[client(1, 0.0, 5.0, 5.0, false)],
            &[],
        )
    }

    #[test]
    fn test_deposit_resolve() -> Result<()> {
        test_process(
            &[tx_deposit(1, 1, 5.0), tx_dispute(1, 1), tx_resolve(1, 1)],
            &[client(1, 5.0, 0.0, 5.0, false)],
            &[],
        )
    }

    #[test]
    fn test_deposit_chargeback() -> Result<()> {
        test_process(
            &[tx_deposit(1, 1, 5.0), tx_dispute(1, 1), tx_chargeback(1, 1)],
            &[client(1, 0.0, 0.0, 0.0, true)],
            &[],
        )
    }

    #[test]
    fn test_withdrawal_dispute() -> Result<()> {
        test_process(
            &[
                tx_deposit(1, 1, 5.0),
                tx_withdrawal(1, 2, 5.0),
                tx_dispute(1, 2),
            ],
            &[client(1, 5.0, -5.0, 0.0, false)],
            &[],
        )
    }

    #[test]
    fn test_withdrawal_resolve() -> Result<()> {
        test_process(
            &[
                tx_deposit(1, 1, 5.0),
                tx_withdrawal(1, 2, 5.0),
                tx_dispute(1, 2),
                tx_resolve(1, 2),
            ],
            &[client(1, 0.0, 0.0, 0.0, false)],
            &[],
        )
    }

    #[test]
    fn test_withdrawal_chargeback() -> Result<()> {
        test_process(
            &[
                tx_deposit(1, 1, 5.0),
                tx_withdrawal(1, 2, 5.0),
                tx_dispute(1, 2),
                tx_chargeback(1, 2),
            ],
            &[client(1, 5.0, 0.0, 5.0, true)],
            &[],
        )
    }

    #[test]
    fn test_withdrawal_dispute_withdrawal_fail() -> Result<()> {
        test_process(
            &[
                tx_deposit(1, 1, 5.0),
                tx_withdrawal(1, 2, 5.0),
                tx_dispute(1, 2),
                tx_withdrawal(1, 3, 5.0),
            ],
            &[client(1, 5.0, -5.0, 0.0, false)],
            &[Message::NotEnoughFunds(1, 3, TxType::Withdrawal)],
        )
    }

    mod helper {
        use crate::{
            client::Client, common::*, message::Message, process::process, transaction::*,
        };
        use anyhow::Result;

        pub fn test_process(
            transactions: &[Tx],
            expected_clients: &[Client],
            expected_messages: &[Message],
        ) -> Result<()> {
            let mut messages = vec![];
            let clients = process(transactions, &mut messages)?;

            assert_eq!(expected_messages, messages, "messages");
            assert_eq!(expected_clients, clients, "clients");

            Ok(())
        }

        pub fn tx_deposit(client_id: ClientId, tx_id: TxId, amount: Value) -> Tx {
            tx(TxType::Deposit, client_id, tx_id, amount)
        }

        pub fn tx_withdrawal(client_id: ClientId, tx_id: TxId, amount: Value) -> Tx {
            tx(TxType::Withdrawal, client_id, tx_id, amount)
        }

        pub fn tx_dispute(client_id: ClientId, tx_id: TxId) -> Tx {
            tx(TxType::Dispute, client_id, tx_id, 0.0)
        }

        pub fn tx_resolve(client_id: ClientId, tx_id: TxId) -> Tx {
            tx(TxType::Resolve, client_id, tx_id, 0.0)
        }

        pub fn tx_chargeback(client_id: ClientId, tx_id: TxId) -> Tx {
            tx(TxType::Chargeback, client_id, tx_id, 0.0)
        }

        fn tx(ty: TxType, client_id: ClientId, tx_id: TxId, amount: Value) -> Tx {
            Tx {
                ty,
                client_id,
                tx_id,
                amount,
                state: Default::default(),
            }
        }

        pub fn client(
            client: ClientId,
            available: Value,
            held: Value,
            total: Value,
            locked: bool,
        ) -> Client {
            Client {
                id: client,
                available,
                held,
                total,
                locked,
                ..Default::default()
            }
        }
    }
}
