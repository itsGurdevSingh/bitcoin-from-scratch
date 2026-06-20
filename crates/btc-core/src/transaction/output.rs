use crate::{script::Script, serialization::BitcoinSerialize};

#[derive(Clone)]
pub struct TxOutput {
    pub value: u64,
    pub script_pub_key: Script,
}

impl BitcoinSerialize for TxOutput {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(
            &self.value.to_le_bytes(),
        );

        let script_bytes =
            self.script_pub_key.serialize();

        bytes.extend_from_slice(
            &(script_bytes.len() as u32).to_le_bytes(),
        );

        bytes.extend(script_bytes);

        bytes
    }
}