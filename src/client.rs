use crate::{data::*, transaction::*};
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

    pub fn process(&mut self, tx: &Tx) {
        if !self.validate(tx) {
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
                    eprintln!(
                        "ERROR: Cannot process withdrawal transaction {} for client {}. Not enough funds.",
                        tx.tx_id, self.id
                    );
                    return;
                }

                self.available -= tx.amount;
                self.total -= tx.amount;
                self.transactions.push(tx.clone());
            }
            TxType::Dispute => match self.transactions.iter_mut().find(|t| t.tx_id == tx.tx_id) {
                Some(t) => match t.state {
                    TxState::Active => {
                        self.available -= tx.amount;
                        self.held += tx.amount;
                        t.state = TxState::InDispute;
                    }
                    TxState::InDispute => {
                        eprintln!(
                            "ERROR: Cannot process dispute transaction {} for client {}. Transaction already in dispute.",
                            tx.client_id, self.id
                        );
                        return;
                    }
                    TxState::Disputed => {
                        eprintln!(
                            "ERROR: Cannot process dispute transaction {} for client {}. Transaction was already disputed.",
                            tx.client_id, self.id
                        );
                        return;
                    }
                },
                None => {
                    eprintln!(
                        "ERROR: Cannot process dispute transaction {} for client {}. Transaction is unknown.",
                        tx.client_id, self.id
                    );
                    return;
                }
            },
            TxType::Resolve => match self.transactions.iter_mut().find(|t| t.tx_id == tx.tx_id) {
                Some(t) => match t.state {
                    TxState::Active => {
                        eprintln!(
                            "ERROR: Cannot process resolve transaction {} for client {}. Transaction is not in dispute.",
                            tx.client_id, self.id
                        );
                        return;
                    }
                    TxState::InDispute => {
                        self.available += tx.amount;
                        self.held -= tx.amount;
                        t.state = TxState::Disputed;
                    }
                    TxState::Disputed => {
                        eprintln!(
                            "ERROR: Cannot process resolve transaction {} for client {}. Transaction was already disputed.",
                            tx.client_id, self.id
                        );
                        return;
                    }
                },
                None => {
                    eprintln!(
                        "ERROR: Cannot process resolve transaction {} for client {}. Transaction is unknown.",
                        tx.client_id, self.id
                    );
                    return;
                }
            },
            TxType::Chargeback => {
                match self.transactions.iter_mut().find(|t| t.tx_id == tx.tx_id) {
                    Some(t) => match t.state {
                        TxState::Active => {
                            eprintln!(
                            "ERROR: Cannot process chargeback transaction {} for client {}. Transaction is not in dispute.",
                            tx.client_id, self.id
                        );
                            return;
                        }
                        TxState::InDispute => {
                            self.held -= tx.amount;
                            self.total -= tx.amount;
                            t.state = TxState::Disputed;
                            self.locked = true;
                        }
                        TxState::Disputed => {
                            eprintln!(
                            "ERROR: Cannot process chargeback transaction {} for client {}. Transaction was already disputed.",
                            tx.client_id, self.id
                        );
                            return;
                        }
                    },
                    None => {
                        eprintln!(
                        "ERROR: Cannot process chargeback transaction {} for client {}. Transaction is unknown.",
                        tx.client_id, self.id
                    );
                        return;
                    }
                }
            }
        }

        // eprintln!("INFO: {:?} -> {:?}", tx, self);
    }

    fn validate(&self, tx: &Tx) -> bool {
        assert_eq!(self.id, tx.client_id);

        if self.locked {
            eprintln!(
                "ERROR: Cannot process transaction {} for client {}. Account is locked.",
                tx.client_id, self.id
            );
            return false;
        }

        if (tx.ty == TxType::Deposit || tx.ty == TxType::Withdrawal)
            && self.transactions.iter().any(|t| t.tx_id == tx.tx_id)
        {
            eprintln!(
                "ERROR: Cannot process transaction {} for client {}. Transaction with the same id was already processed.",
                tx.client_id, self.id
            );
            return false;
        }

        true
    }
}
