use crate::serialization::BitcoinSerialize;

#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Witness {
    pub stack: Vec<Vec<u8>>
}

impl Witness {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

impl BitcoinSerialize for Witness {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&(self.stack.len() as u32).to_le_bytes());
        
        for item in &self.stack {
            bytes.extend_from_slice(&(item.len() as u32).to_le_bytes());
            bytes.extend(item);
        };
        bytes
    }
}