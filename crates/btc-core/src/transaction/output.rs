use crate::{
    script::Script, serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxOutput {
    pub value: u64,
    pub script_pub_key: Script,
}

impl BitcoinSerialize for TxOutput {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.value.to_le_bytes());

        let script_bytes = self.script_pub_key.serialize();

        bytes.extend(script_bytes);

        bytes
    }
}

impl BitcoinDeserialize for TxOutput {
    type Error = DeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        if bytes.len() < offset + 8 {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let value = u64::from_le_bytes(
            bytes[offset..offset + 8]
            .try_into()
            .map_err( |_| DeserializeError::InvalidCompactSize)?
        );
        offset += 8;

        let (script, consumed) = Script::deserialize(&bytes[offset..])?;
        offset += consumed;


        Ok((
            Self {
                value,
                script_pub_key: script
            },
            offset
        ))
    }
}
