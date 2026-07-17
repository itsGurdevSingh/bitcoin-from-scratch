use crate::{
    script::{Script, ScriptItem}, transaction::{OutPoint, Transaction, TxInput, TxOutput, Witness}, types::TxId,
};

pub struct CoinBase;

impl CoinBase {
    pub fn create_transaction(
        reward: u64,
        fees: u64,
        height: u32,
        script_pub_key: Script,
    ) -> Transaction {
        Transaction {
            version: 0,
            inputs: vec![TxInput {
                previous_output: OutPoint {
                    txid: TxId([0u8; 32]),
                    vout: 0xffffffff,
                },
                script_sig: Script {
                    items: vec![ScriptItem::PushData(height.to_le_bytes().to_vec())],
                },
                witness: Witness { stack: vec![vec![0u8; 32]] },
                sequence: 0xffffffff,
            }],
            outputs: vec![TxOutput {
                value: reward + fees,
                script_pub_key,
            }],
            lock_time: 0,
        }
    }
}
