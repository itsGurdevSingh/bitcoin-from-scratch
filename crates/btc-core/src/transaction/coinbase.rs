use crate::{
    block::constants::WITNESS_COMMITMENT_HEADER, crypto::sha256d, script::{OpCode, Script, ScriptItem}, transaction::{OutPoint, Transaction, TxInput, TxOutput, Witness}, types::{MerkleRoot, TxId},
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
                witness: Witness { stack: vec![] },
                sequence: 0xffffffff,
            }],
            outputs: vec![TxOutput {
                value: reward + fees,
                script_pub_key,
            }],
            lock_time: 0,
        }
    }

    pub fn create_transaction_witness_v0(
        reward: u64,
        fees: u64,
        height: u32,
        script_pub_key: Script,
        witness_merkle_root: MerkleRoot
    ) -> Transaction {
        let mut merkle_root = witness_merkle_root.into_bytes().to_vec();
        merkle_root.extend([0u8;32]);
        let witness_commitment = sha256d(&merkle_root);
        let mut tx = Self::create_transaction(reward, fees, height, script_pub_key);
        tx.outputs.push(TxOutput {
                value: 0,
                script_pub_key: Script{
                    items:vec![
                    ScriptItem::Op(OpCode::Return),
                    ScriptItem::PushData(WITNESS_COMMITMENT_HEADER.to_vec()),
                    ScriptItem::PushData(witness_commitment.to_vec())
                    ]
                }
            });

        tx.inputs[0].witness.stack.push(vec![0u8; 32]);
        tx
    }
}
