use btc_core::serialization::{BitcoinDeserialize, BitcoinSerialize, compact_size::{get_compact_size, read_compact_size}};

use crate::network::{InventoryVector, NetworkDeserializeError};

#[derive(Debug, Clone)]
pub struct GetDataMessage {
    pub inventory: Vec<InventoryVector>,
}

impl BitcoinSerialize for GetDataMessage {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(get_compact_size(self.inventory.len()));
        
        for vector in self.inventory.iter() {
            bytes.extend(vector.serialize());
        };

        bytes
    }
}

impl BitcoinDeserialize for GetDataMessage {
    type Error = NetworkDeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {

        let mut offset: usize = 0;

        let (inv_len, consumed) = read_compact_size(&bytes).map_err(|_|NetworkDeserializeError::InvalidType)?;
        offset += consumed;

        let mut inventory: Vec<InventoryVector> = Vec::new();
        for _ in 0..inv_len {
            let (vector, consmed) = InventoryVector::deserialize(&bytes[offset..])?;
            inventory.push(vector);
            offset += consmed;
        };

        Ok((Self {
            inventory
        }, offset))
    }
    
}