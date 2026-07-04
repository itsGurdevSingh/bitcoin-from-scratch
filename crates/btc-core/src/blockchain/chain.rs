use crate::{
    block::{Block, BlockHeader, BlockReward}, blockchain::{BlockProcessor, constants::INITIAL_BITS, error::BlockchainError}, ledger::Ledger, mempool::Mempool, miner::Miner, script::{OpCode, Script, ScriptItem}, transaction::{self, CoinBase, Transaction}, types::{BlockHash, MerkleRoot}, utils::time::Time,
};

pub struct Blockchain {
    blocks: Vec<Block>,
    ledger: Ledger,
    mempool: Mempool,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Self::create_genesis();

        Self {
            blocks: vec![genesis],
            ledger: Ledger::new(),
            mempool: Mempool::new(),
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {

        // validate header 
        // is valid previos block hash 
        if block.header.previous_block_hash != self.tip().map_err(|e| e)?.header.hash() {
            return Err(BlockchainError::WrongPreviousBlock);
        };

        // validate
        let states = BlockProcessor::process(&block, &self.ledger)
            .map_err(|e| BlockchainError::Processor(e))?;

        // commit states to ledger
        for state in states.iter() {
            self.ledger
                .commit_state(state)
                .map_err(|e| BlockchainError::Ledger(e))?;
        }

        // remove from mempool
        for tx in block.transactions.iter() {
            if tx.is_coinbase() {
                continue;
            }
            self.mempool
                .remove_transaction(&tx.txid())
                .ok_or(BlockchainError::Mempool)?;
        }

        self.blocks.push(block);

        Ok(())
    }

    pub fn tip(&self) -> Result<&Block, BlockchainError> {
        let block = self.blocks.last().ok_or(BlockchainError::ChainIsEmpty)?;
        Ok(block)
    }

    pub fn height(&self) -> u32 {
        self.blocks.len() as u32
    }

    pub fn create_genesis() -> Block {

        let reward = BlockReward::subsidy(0);

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

        let transaction = CoinBase::create_transaction(reward, 0, 0, script);

        let mut block = Block {
            header: BlockHeader {
                version: 0,
                previous_block_hash: BlockHash([0u8; 32]),
                merkle_root: MerkleRoot([0u8; 32]),
                timestamp: Time::unix_timestamp(),
                bits: INITIAL_BITS,
                nonce: 0,
            },
            transactions: vec![transaction],
        };
        let _ = Miner::mine(&mut block);

        block
    }

}
