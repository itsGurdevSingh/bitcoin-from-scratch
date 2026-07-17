use crate::script::Script;
use crate::serialization::BitcoinSerialize;
use crate::serialization::compact_size::get_compact_size;
use crate::transaction::{OutPoint, Witness};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub witness: Witness,
    pub sequence: u32,
}

impl BitcoinSerialize for TxInput {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.previous_output.serialize());

        let script_bytes = self.script_sig.serialize();

        bytes.extend(get_compact_size(script_bytes.len()));

        bytes.extend(script_bytes);

        bytes.extend_from_slice(&self.sequence.to_le_bytes());

        bytes
    }
}
