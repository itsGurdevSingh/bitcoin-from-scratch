use crate::serialization::{
    BitcoinDeserialize, BitcoinSerialize, DeserializeError,
    compact_size::{get_compact_size, read_compact_size},
};

#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Witness {
    pub stack: Vec<Vec<u8>>,
}

impl Witness {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

impl BitcoinSerialize for Witness {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(get_compact_size(self.stack.len()));

        for item in &self.stack {
            bytes.extend(get_compact_size(item.len()));
            bytes.extend(item);
        }
        bytes
    }
}

impl BitcoinDeserialize for Witness {
    type Error = DeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        if bytes.is_empty() {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let mut offset = 0;

        // number of stack items
        let (count, consumed) = read_compact_size(&bytes[offset..])?;
        offset += consumed;

        let mut stack = Vec::with_capacity(count);

        for _ in 0..count {
            // length of this stack item
            let (len, consumed) = read_compact_size(&bytes[offset..])?;
            offset += consumed;

            if bytes.len() < offset + len {
                return Err(DeserializeError::UnexpectedEndOfBytes);
            }

            // copy bytes
            let item = bytes[offset..offset + len].to_vec();
            offset += len;

            stack.push(item);
        }

        Ok((Witness { stack }, offset))
    }
}
