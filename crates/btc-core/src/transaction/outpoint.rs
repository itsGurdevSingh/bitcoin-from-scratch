use crate::types::TxId;

#[derive(Eq, Hash, PartialEq)]
pub struct OutPoint {
    pub txid: TxId,
    pub vout: u32
}