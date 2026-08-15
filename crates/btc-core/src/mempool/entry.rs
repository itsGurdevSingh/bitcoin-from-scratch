use crate::{
    serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError},
    transaction::Transaction,
};

pub struct MempoolEntry {
    pub tx: Transaction,
    pub fee: u64,
}

impl MempoolEntry {
    pub fn new() -> Self {
        Self { tx: Transaction::new(), fee: 0 }
    }
}

impl BitcoinSerialize for MempoolEntry {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend(self.tx.serialize_witness());
        bytes.extend(self.fee.to_le_bytes());

        bytes
    }
}

impl BitcoinDeserialize for MempoolEntry {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        let (tx, consumed) = Transaction::deserialize_witness(bytes)?;
        offset += consumed;

        let fee = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 8;
        Ok((Self { tx, fee }, offset))
    }
}
