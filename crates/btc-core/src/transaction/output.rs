use crate::{
    script::Script,
    serialization::{BitcoinSerialize, compact_size::get_compact_size},
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

        bytes.extend(get_compact_size(script_bytes.len()));

        bytes.extend(script_bytes);

        bytes
    }
}

impl TxOutput {
    pub fn validate() {}
}
