use crate::common::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub enum TxType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

pub type TxId = u32;

#[derive(Deserialize)]
pub struct InputTx {
    #[serde(rename = "type")]
    pub ty: TxType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum TxState {
    Active,
    InDispute,
    Disputed,
}

impl Default for TxState {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone)]
pub struct Tx {
    pub ty: TxType,
    pub client_id: ClientId,
    pub tx_id: TxId,
    pub amount: Value,
    pub state: TxState,
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
