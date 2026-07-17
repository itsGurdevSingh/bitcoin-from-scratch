use crate::{
    serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError},
    types::TxId,
};

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct OutPoint {
    pub txid: TxId,
    pub vout: u32,
}

impl BitcoinSerialize for OutPoint {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.txid.0);

        bytes.extend_from_slice(&self.vout.to_le_bytes());

        bytes
    }
}

impl BitcoinDeserialize for OutPoint {
    type Error = DeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        if bytes.len() < 36 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let mut offset = 0;

        let txid = TxId(
            bytes[..offset + 32]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 32;

        let vout = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 4;

        Ok((Self { txid, vout }, offset))
    }
}
