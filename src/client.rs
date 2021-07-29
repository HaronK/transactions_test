use crate::{common::*, message::Message, transaction::*};
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Client {
    #[serde(rename = "client")]
    pub id: ClientId,
    pub available: Value,
    pub held: Value,
    pub total: Value,
    pub locked: bool,

    /// Client's deposit and withdrawal transactions
    #[serde(skip)]
    pub transactions: Vec<Tx>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("id", &self.id)
            .field("a", &self.available)
            .field("h", &self.held)
            .field("t", &self.total)
            .field("l", &self.locked)
            .finish()
    }
}

impl Client {
    pub fn new(client: ClientId) -> Self {
        Self {
            id: client,
            ..Default::default()
        }
    }

    pub fn process(&mut self, tx: &Tx, messages: &mut Vec<Message>) {
        if !self.validate(tx, messages) {
            return;
        }

        match tx.ty {
            TxType::Deposit => {
                self.available += tx.amount;
                self.total += tx.amount;
                self.transactions.push(tx.clone());
            }
            TxType::Withdrawal => {
                if self.available < tx.amount {
                    messages.push(Message::NotEnoughFunds(tx.client_id, tx.tx_id, tx.ty));
                } else {
                    self.available -= tx.amount;
                    self.total -= tx.amount;
                    self.transactions.push(tx.clone());
                }
            }
            TxType::Dispute => match self.transactions.iter_mut().find(|t| t.tx_id == tx.tx_id) {
                Some(t) => match t.state {
                    TxState::Active => {
                        self.available -= tx.amount;
                        self.held += tx.amount;
                        t.state = TxState::InDispute;
                    }
                    TxState::InDispute => {
                        messages.push(Message::AlreadyInDispute(tx.client_id, tx.tx_id, tx.ty));
                    }
                    TxState::Disputed => {
                        messages.push(Message::AlreadyDisputed(tx.client_id, tx.tx_id, tx.ty));
                    }
                },
                None => {
                    messages.push(Message::UnknownTransaction(tx.client_id, tx.tx_id, tx.ty));
                }
            },
            TxType::Resolve => match self.transactions.iter_mut().find(|t| t.tx_id == tx.tx_id) {
                Some(t) => match t.state {
                    TxState::Active => {
                        messages.push(Message::NotInDispute(tx.client_id, tx.tx_id, tx.ty));
                    }
                    TxState::InDispute => {
                        self.available += tx.amount;
                        self.held -= tx.amount;
                        t.state = TxState::Disputed;
                    }
                    TxState::Disputed => {
                        messages.push(Message::AlreadyDisputed(tx.client_id, tx.tx_id, tx.ty));
                    }
                },
                None => {
                    messages.push(Message::UnknownTransaction(tx.client_id, tx.tx_id, tx.ty));
                }
            },
            TxType::Chargeback => {
                match self.transactions.iter_mut().find(|t| t.tx_id == tx.tx_id) {
                    Some(t) => match t.state {
                        TxState::Active => {
                            messages.push(Message::NotInDispute(tx.client_id, tx.tx_id, tx.ty));
                        }
                        TxState::InDispute => {
                            self.held -= tx.amount;
                            self.total -= tx.amount;
                            t.state = TxState::Disputed;
                            self.locked = true;
                        }
                        TxState::Disputed => {
                            messages.push(Message::AlreadyDisputed(tx.client_id, tx.tx_id, tx.ty));
                        }
                    },
                    None => {
                        messages.push(Message::UnknownTransaction(tx.client_id, tx.tx_id, tx.ty));
                    }
                }
            }
        }

        // eprintln!("INFO: {:?} -> {:?}", tx, self);
    }

    fn validate(&self, tx: &Tx, messages: &mut Vec<Message>) -> bool {
        assert_eq!(self.id, tx.client_id);

        if self.locked {
            messages.push(Message::AccountIsLocked(tx.client_id, tx.tx_id, tx.ty));
            return false;
        }

        if (tx.ty == TxType::Deposit || tx.ty == TxType::Withdrawal)
            && self.transactions.iter().any(|t| t.tx_id == tx.tx_id)
        {
            messages.push(Message::TransactionExist(tx.client_id, tx.tx_id, tx.ty));
            return false;
        }

        true
    }
}
