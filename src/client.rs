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

    /// Client's transaction indices in the global list of transactions (arena)
    #[serde(skip)]
    pub transactions: Vec<usize>,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.available == other.available
            && self.held == other.held
            && self.total == other.total
            && self.locked == other.locked
    }
}

impl PartialEq<&Client> for Client {
    fn eq(&self, other: &&Client) -> bool {
        self.eq(*other)
    }
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
    pub fn new(id: ClientId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn process<'a, F>(&mut self, tx_idx: usize, mut get_tx: F, messages: &mut Vec<Message>)
    where
        F: FnMut(usize) -> Option<&'a mut Tx>,
    {
        // Incoming transaction index should always correspond to the valid transaction object
        let tx = get_tx(tx_idx).unwrap();

        if !self.validate(tx_idx, tx, messages) {
            return;
        }

        match tx.ty {
            TxType::Deposit => {
                self.available += tx.amount;
                self.total += tx.amount;
                // self.transactions.push(tx);
            }
            TxType::Withdrawal => {
                if self.available < tx.amount || self.total < tx.amount {
                    messages.push(Message::NotEnoughFunds(tx.client_id, tx.tx_id, tx.ty));
                } else {
                    self.available -= tx.amount;
                    self.total -= tx.amount;
                    // self.transactions.push(tx);
                }
            }
            TxType::Dispute => match self.get_tx_by_id(tx.tx_id, get_tx) {
                Some(t) => match t.state {
                    TxState::Active => {
                        let amount = t.dispute_amount();
                        self.available -= amount;
                        self.held += amount;
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
            TxType::Resolve => match self.get_tx_by_id(tx.tx_id, get_tx) {
                Some(t) => match t.state {
                    TxState::Active => {
                        messages.push(Message::NotInDispute(tx.client_id, tx.tx_id, tx.ty));
                    }
                    TxState::InDispute => {
                        let amount = t.dispute_amount();
                        self.available += amount;
                        self.held -= amount;
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
            TxType::Chargeback => match self.get_tx_by_id(tx.tx_id, get_tx) {
                Some(t) => match t.state {
                    TxState::Active => {
                        messages.push(Message::NotInDispute(tx.client_id, tx.tx_id, tx.ty));
                    }
                    TxState::InDispute => {
                        let amount = t.dispute_amount();
                        self.held -= amount;
                        self.total -= amount;
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
            },
        }

        self.transactions.push(tx_idx);

        // eprintln!("INFO: {:?} -> {:?}", tx, self);
    }

    fn get_tx_by_id<'a, F>(&self, tx_id: TxId, mut get_tx: F) -> Option<&'a mut Tx>
    where
        F: FnMut(usize) -> Option<&'a mut Tx>,
    {
        self.transactions
            .iter()
            .map(|tx_idx| get_tx(*tx_idx))
            .filter(|opt_tx| opt_tx.is_some())
            .map(|opt_tx| opt_tx.unwrap())
            .find(|tx| tx.tx_id == tx_id)
    }

    fn validate(&self, tx_idx: usize, tx: &Tx, messages: &mut Vec<Message>) -> bool {
        assert_eq!(self.id, tx.client_id);

        if self.locked {
            messages.push(Message::AccountIsLocked(tx.client_id, tx.tx_id, tx.ty));
            return false;
        }

        if self.transactions.iter().any(|idx| *idx == tx_idx) {
            messages.push(Message::TransactionExist(tx.client_id, tx.tx_id, tx.ty));
            return false;
        }

        true
    }
}
