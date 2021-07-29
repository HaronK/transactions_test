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

impl Tx {
    pub fn dispute_amount(&self) -> Value {
        match self.ty {
            TxType::Deposit => self.amount,
            TxType::Withdrawal => -self.amount,
            TxType::Dispute => unreachable!(),
            TxType::Resolve => unreachable!(),
            TxType::Chargeback => unreachable!(),
        }
    }
}
