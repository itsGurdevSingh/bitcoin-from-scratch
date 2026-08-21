use btc_core::serialization::{BitcoinDeserialize, BitcoinSerialize};

use crate::network::NetworkDeserializeError;

pub struct PingMessage {
    pub nonce: u64,
}
pub struct PongMessage {
    pub nonce: u64,
}


impl BitcoinSerialize for PingMessage {
    fn serialize(&self) -> Vec<u8> {
        self.nonce.to_le_bytes().to_vec()
    }
}
impl BitcoinSerialize for PongMessage {
    fn serialize(&self) -> Vec<u8> {
        self.nonce.to_le_bytes().to_vec()
    }
}

impl BitcoinDeserialize for PingMessage {
    type Error = NetworkDeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        Ok((
            Self {
                nonce: u64::from_le_bytes(bytes.try_into().map_err(|_|NetworkDeserializeError::InvalidType)?)
            }, 8
        ))
    }
}
impl BitcoinDeserialize for PongMessage {
    type Error = NetworkDeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        Ok((
            Self {
                nonce: u64::from_le_bytes(bytes.try_into().map_err(|_|NetworkDeserializeError::InvalidType)?)
            }, 8
        ))
    }
}