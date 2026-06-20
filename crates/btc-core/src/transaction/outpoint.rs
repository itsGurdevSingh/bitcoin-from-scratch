use crate::types::TxId;

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct OutPoint {
    pub txid: TxId,
    pub vout: u32
}