use secp256k1::XOnlyPublicKey;

use crate::serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LeafVesrion {
    V1 = 0xC0
}
impl LeafVesrion {
    fn from_u8(byte: u8) -> Option<Self> {
        match byte {
        0xC0 => Some(Self::V1),
        _ => None
        }
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlBlock {
    pub parity: bool,
    pub leaf_version: LeafVesrion,
    pub internal_key: XOnlyPublicKey,
    pub merkle_path: Vec<[u8; 32]>,
}

impl BitcoinSerialize for ControlBlock {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let control_byte = self.leaf_version as u8 | (self.parity as u8);
        bytes.push(control_byte);
        bytes.extend(self.internal_key.serialize());

        for path in self.merkle_path.iter() {
            bytes.extend_from_slice(path);
        }

        bytes
    }
}

impl BitcoinDeserialize for ControlBlock {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {

        if bytes.len() < 33 {
            Err(DeserializeError::UnexpectedEndOfBytes)?;
        }
        if (bytes.len() - 33) % 32 != 0 {
            Err(DeserializeError::UnexpectedEndOfBytes)?
        }

        let mut consumed: usize = 0;
        let parity = (bytes[consumed] & 1) == 1;

        let leaf_version = LeafVesrion::from_u8(bytes[consumed] & 0xFE).ok_or(DeserializeError::UnexpectedEndOfBytes)?;
        consumed += 1;
        let mut internal_key_bytes = [0u8; 32];

        internal_key_bytes.copy_from_slice(&bytes[consumed..consumed + 32]);
        consumed += 32;

        let internal_key = XOnlyPublicKey::from_byte_array(internal_key_bytes)
            .map_err(|_| DeserializeError::UnexpectedEndOfBytes)?;

        let mut merkle_path = Vec::new();

        for _ in 0..((bytes.len() - consumed) / 32) {
            let mut a = [0u8; 32];
            a.copy_from_slice(&bytes[consumed..consumed + 32]);
            consumed += 32;
            merkle_path.push(a);
        }
        Ok((
            Self {
                parity,
                leaf_version,
                internal_key,
                merkle_path,
            },
            consumed,
        ))
    }
}


#[cfg(test)]
mod test {

    use std::assert_eq;

use super::*;

    #[test]
    fn serlize_then_deselize_result_same() {
        let control_block = ControlBlock {
            parity: true,
            leaf_version: LeafVesrion::V1,
            internal_key: XOnlyPublicKey::from_byte_array([1u8;32]).unwrap(),
            merkle_path: vec![[0u8;32], [1u8;32], [2u8; 32]]
        };

        
        let ser_block = control_block.serialize();

        let (der_block, _) = ControlBlock::deserialize(&ser_block).unwrap();

        assert_eq!(control_block, der_block);
    }
}