use crate::{
    script::OpCode,
    serialization::{
        BitcoinDeserialize, BitcoinSerialize, DeserializeError,
        compact_size::{get_compact_size, read_compact_size},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptItem {
    Op(OpCode),
    PushData(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub items: Vec<ScriptItem>,
}

impl ScriptItem {
    pub fn get_bytes(&self) -> Option<&[u8]> {
        match self {
            ScriptItem::PushData(data) => Some(data),
            ScriptItem::Op(_) => None,
        }
    }
}

impl Script {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn sig_op_cost(&self) -> u32 {
        let mut cost: u32 = 0;
        for item in self.items.iter() {
            match item {
                ScriptItem::Op(op) => match op {
                    OpCode::CheckSig | OpCode::CheckSigVerify | OpCode::CheckSigAdd => cost += 1,
                    _ => {}
                },
                _ => {}
            }
        }
        cost
    }
}

impl Extend<ScriptItem> for Script {
    fn extend<T: IntoIterator<Item = ScriptItem>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl BitcoinSerialize for ScriptItem {
    fn serialize(&self) -> Vec<u8> {
        match self {
            ScriptItem::Op(op) => op.serialize(),

            ScriptItem::PushData(data) => {
                let mut bytes = Vec::new();
                bytes.push(0x01); // we will add reserved value for push operation because we are using different structure.

                bytes.extend(get_compact_size(data.len()));

                bytes.extend_from_slice(data);

                bytes
            }
        }
    }
}

impl BitcoinDeserialize for ScriptItem {
    type Error = DeserializeError;

    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        if bytes.is_empty() {
            return Err(DeserializeError::UnexpectedEndOfBytes);
        }

        let mut offset: usize = 0;
        match bytes[0] {
            0x01 => {
                offset += 1;
                let (len, consumed) = read_compact_size(&bytes[offset..])?;

                offset += consumed;
                if bytes.len() < offset + len {
                    return Err(DeserializeError::UnexpectedEndOfBytes);
                }

                let data = bytes[offset..offset + len].to_vec();
                offset += len;

                Ok((ScriptItem::PushData(data), offset))
            } // push op
            _ => {
                let (op, consumed) = OpCode::deserialize(bytes)?;
                Ok((ScriptItem::Op(op), consumed))
            }
        }
    }
}

impl BitcoinSerialize for Script {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(get_compact_size(self.items.len()));

        for item in &self.items {
            bytes.extend(item.serialize());
        }

        bytes
    }
}

impl BitcoinDeserialize for Script {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        let (len, consumed) = read_compact_size(bytes)?;
        offset += consumed;

        let mut items: Vec<ScriptItem> = vec![];
        for _ in 0..len {
            let (item, consumed) = ScriptItem::deserialize(&bytes[offset..])?;
            items.push(item);
            offset += consumed;
        }

        Ok((Self { items }, offset))
    }
}
