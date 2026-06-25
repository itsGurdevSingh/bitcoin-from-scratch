use crate::{serialization::BitcoinSerialize, types::TxId};

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct OutPoint {
    pub txid: TxId,
    pub vout: u32
}

impl BitcoinSerialize for OutPoint {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.txid.0);

        bytes.extend_from_slice(&self.vout.to_le_bytes());

        bytes
    }
}