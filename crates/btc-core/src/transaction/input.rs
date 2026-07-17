use crate::script::Script;
use crate::serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError};
use crate::transaction::{OutPoint, Witness};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub witness: Witness,
    pub sequence: u32,
}

impl BitcoinSerialize for TxInput {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.previous_output.serialize());

        bytes.extend(self.script_sig.serialize());

        bytes.extend_from_slice(&self.sequence.to_le_bytes());

        bytes
    }
}

impl BitcoinDeserialize for TxInput {
    type Error = DeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        let (previous_output, consumed) = OutPoint::deserialize(&bytes[offset..])?;
        offset += consumed;
        let (script_sig, consumed) = Script::deserialize(&bytes[offset..])?;
        offset += consumed;

        if bytes.len() < offset + 4 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let sequence = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| DeserializeError::InvalidCompactSize)?,
        );
        offset += 4;

        Ok((
            Self {
                previous_output,
                script_sig,
                sequence,
                witness: Witness::new(),
            },
            offset,
        ))
    }
}
