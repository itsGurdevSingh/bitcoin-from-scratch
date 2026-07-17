use crate::{
    crypto::sha256d, serialization::{BitcoinSerialize, compact_size::get_compact_size}, transaction::{TxInput, TxOutput, Witness}, types::{TxId, WTxId},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u64,
}

impl BitcoinSerialize for Transaction {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.version.to_le_bytes());

        bytes.extend(get_compact_size(self.inputs.len()));

        for input in &self.inputs {
            bytes.extend(input.serialize());
        }

        bytes.extend(get_compact_size(self.outputs.len()));

        for output in &self.outputs {
            bytes.extend(output.serialize());
        }

        bytes.extend_from_slice(&self.lock_time.to_le_bytes());

        bytes
    }
}

impl Transaction {
    pub fn serialize_witness(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.version.to_le_bytes());

        bytes.push(0x00); // marker
        bytes.push(0x01); // flag

        bytes.extend(get_compact_size(self.inputs.len()));

        for input in &self.inputs {
            bytes.extend(input.serialize());
        }

        bytes.extend(get_compact_size(self.outputs.len()));

        for output in &self.outputs {
            bytes.extend(output.serialize());
        }

        for input in &self.inputs {
            bytes.extend(input.witness.serialize());
        }

        bytes.extend_from_slice(&self.lock_time.to_le_bytes());

        bytes
    }
}

impl Transaction {
    pub fn txid(&self) -> TxId {
        let bytes = self.serialize();
        TxId::from(sha256d(&bytes))
    }

    pub fn wtid(&self) -> WTxId {
        let bytes = self.serialize_witness();
        WTxId::from(sha256d(&bytes))
    }

    pub fn signing_hash(&self) -> [u8; 32] {
        let mut clone = self.clone();
        for input in clone.inputs.iter_mut() {
            input.script_sig.items = vec![];
            input.witness = Witness { stack: vec![] }
        }

        let serial = clone.serialize();
        sha256d(&serial)
    }

    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1
            && self.inputs[0].previous_output.txid == TxId([0u8; 32])
            && self.inputs[0].previous_output.vout == 0xffffffff
    }
}

#[cfg(test)]
mod test {

    use crate::{
        script::{OpCode, Script, ScriptItem}, transaction::{OutPoint, Witness},
    };

    use super::*;

    #[test]
    fn same_transaction_same_txid() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let res = transaction.txid();

        let res2 = transaction.txid();

        assert_eq!(res, res2);
    }

    #[test]
    fn different_transactions_different_txids() {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output();

        let transaction = Transaction {
            version: 10,
            inputs: vec![tx_input.clone()],
            outputs: vec![tx_output.clone()],
            lock_time: 1000,
        };
        let transaction2 = Transaction {
            version: 5,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        let res1 = transaction.txid();

        let res2 = transaction2.txid();

        assert_ne!(res1, res2);
    }

    fn create_dummy_tx_input() -> TxInput {
        let sig_script_items: Vec<ScriptItem> = vec![
            ScriptItem::PushData(vec![0u8; 32]),
            ScriptItem::PushData(vec![0u8; 64]),
        ];

        let script_sig = Script {
            items: sig_script_items,
        };

        let previous_output = OutPoint {
            txid: TxId([0u8; 32]),
            vout: 8,
        };

        TxInput {
            previous_output,
            script_sig,
            witness: Witness { stack: Vec::new() },
            sequence: 5,
        }
    }

    fn create_dummy_tx_output() -> TxOutput {
        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(vec![0u8; 20]), // 20-byte dummy pubkey hash
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        let script: Script = Script {
            items: p2pkh_script,
        };

        TxOutput {
            value: 2,
            script_pub_key: script,
        }
    }
}
