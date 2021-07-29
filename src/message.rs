use crate::{common::*, transaction::*};

#[derive(PartialEq)]
pub enum Message {
    NotEnoughFunds(ClientId, TxId, TxType),
    AlreadyInDispute(ClientId, TxId, TxType),
    AlreadyDisputed(ClientId, TxId, TxType),
    NotInDispute(ClientId, TxId, TxType),
    AccountIsLocked(ClientId, TxId, TxType),
    TransactionExist(ClientId, TxId, TxType),
    UnknownTransaction(ClientId, TxId),
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut get_msg = |ty, tx, c, msg| {
            f.write_fmt(format_args!(
                "ERROR: Cannot process {:?} transaction {} for client {}. {}.",
                ty, tx, c, msg
            ))
        };

        match self {
            Self::NotEnoughFunds(c, tx, ty) => get_msg(ty, tx, c, "Not enough funds"),
            Self::AlreadyInDispute(c, tx, ty) => {
                get_msg(ty, tx, c, "Transaction already in dispute")
            }
            Self::AlreadyDisputed(c, tx, ty) => {
                get_msg(ty, tx, c, "Transaction was already disputed")
            }
            Self::NotInDispute(c, tx, ty) => get_msg(ty, tx, c, "Transaction is not in dispute"),
            Self::AccountIsLocked(c, tx, ty) => get_msg(ty, tx, c, "Account is locked"),
            Self::TransactionExist(c, tx, ty) => get_msg(
                ty,
                tx,
                c,
                "Transaction with the same id was already processed",
            ),
            Self::UnknownTransaction(c, tx) => f.write_fmt(format_args!(
                "ERROR: Cannot process transaction {} for client {}. Transaction is unknown.",
                tx, c
            )),
        }
    }
}
