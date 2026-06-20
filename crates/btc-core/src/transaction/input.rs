use crate::script::Script;
use crate::serialization::BitcoinSerialize;
use crate::transaction::OutPoint;

#[derive(Clone)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl BitcoinSerialize for TxInput {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.previous_output.serialize());

        let script_bytes = self.script_sig.serialize();

        bytes.extend_from_slice(
            &(script_bytes.len() as u32).to_le_bytes(),
        );

        bytes.extend(script_bytes);

        bytes.extend_from_slice(
            &self.sequence.to_le_bytes(),
        );

        bytes
    }
}