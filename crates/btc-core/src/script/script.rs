use crate::{script::OpCode, serialization::BitcoinSerialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptItem {
    Op(OpCode),
    PushData(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub items: Vec<ScriptItem>,
}


impl BitcoinSerialize for ScriptItem {
    fn serialize(&self) -> Vec<u8> {
        match self {
            ScriptItem::Op(op) => op.serialize(),

            ScriptItem::PushData(data) => {
                let mut bytes = Vec::new();

                bytes.push(data.len() as u8);

                bytes.extend_from_slice(data);

                bytes
            }
        }
    }
}


impl BitcoinSerialize for Script {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for item in &self.items {
            bytes.extend(item.serialize());
        }

        bytes
    }
}