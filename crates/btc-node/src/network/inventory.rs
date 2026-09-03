use btc_core::serialization::{BitcoinDeserialize, BitcoinSerialize};

use crate::network::{NetworkDeserializeError, NetworkError};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum InventoryType {
    Tx,
    Block,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct InventoryVector {
    pub inv_type: InventoryType,
    pub hash: [u8; 32],
}

impl InventoryVector {
    pub fn new(inv_type: InventoryType, hash: [u8; 32]) -> Self {
        Self { inv_type, hash }
    }
}

impl BitcoinSerialize for InventoryVector {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.push(self.inv_type.clone() as u8);
        bytes.extend(self.hash);

        bytes
    }
}

impl BitcoinDeserialize for InventoryVector {
    type Error = NetworkDeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        if bytes.len() < 33 {
            Err(NetworkDeserializeError::UnexpectedEndOfBytes {
                expected: 33,
                actual: bytes.len(),
            })?;
        };

        let vector = Self {
            inv_type: InventoryType::try_from(bytes[0])
                .map_err(|_| NetworkDeserializeError::InvalidType)?,
            hash: bytes[1..33]
                .try_into()
                .map_err(|_| NetworkDeserializeError::InvalidType)?,
        };

        Ok((vector, 33))
    }
}

impl TryFrom<u8> for InventoryType {
    type Error = NetworkError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Tx),
            1 => Ok(Self::Block),
            _ => Err(NetworkError::TypeCastFailed),
        }
    }
}
