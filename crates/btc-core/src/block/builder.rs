use crate::{
    block::{Block, BlockHeader, BlockReward, BuilderErrors},
    blockchain::Blockchain,
    merkle::MerkleTree,
    script::Script,
    transaction::{CoinBase, Transaction},
    utils::time::Time,
};

pub struct Builder;

impl Builder {
    pub fn build(
        transactions: &[Transaction],
        miner_script_pub_key: Script,
        chain: &Blockchain,
    ) -> Result<Block, BuilderErrors> {

        
        let mut txs: Vec<Transaction> = Vec::with_capacity(transactions.len() + 1);
        let coinbase = Self::build_coinbase_tx(transactions, miner_script_pub_key, chain)?;

        txs.push(coinbase);
        txs.extend_from_slice(transactions);

        Ok(Block {
            header: BlockHeader {
                version: 1,
                previous_block_hash: chain
                    .tip_node()
                    .map_err(|e| BuilderErrors::Chain(e))?
                    .hash,
                merkle_root: MerkleTree::compute_root(&txs)
                    .map_err(|_| BuilderErrors::InvalidMerkleRoot)?,
                timestamp: Time::unix_timestamp(),
                bits: chain.expected_bits().map_err(|e| BuilderErrors::Chain(e))?,
                nonce: 0,
            },
            transactions: txs,
        })
    }

    fn build_coinbase_tx(
        txs: &[Transaction],
        miner_script_pub_key: Script,
        chain: &Blockchain,
    ) -> Result<Transaction, BuilderErrors> {
        let mut total_input = 0;
        let mut total_output = 0;
        for tx in txs {
            for input in tx.inputs.iter() {
                total_input += chain
                    .ledger()
                    .get_utxo(&input.previous_output)
                    .ok_or(BuilderErrors::InvalidTxs)?
                    .value;
            }

            for output in tx.outputs.iter() {
                total_output += output.value;
            }
        }

        if total_input < total_output {
            return Err(BuilderErrors::InvalidTxs);
        }

        let fees = total_input - total_output;

        let reward = BlockReward::subsidy(chain.height());

        Ok(CoinBase::create_transaction(
            reward,
            fees,
            chain.height() + 1,
            miner_script_pub_key,
        ))
    }
}
