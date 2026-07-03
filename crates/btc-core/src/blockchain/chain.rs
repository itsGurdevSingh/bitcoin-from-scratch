use crate::{
    block::Block,
    blockchain::{BlockProcessor, error::BlockchainError},
    ledger::Ledger,
    mempool::Mempool,
};

pub struct Blockchain {
    blocks: Vec<Block>,
    ledger: Ledger,
    mempool: Mempool,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            ledger: Ledger::new(),
            mempool: Mempool::new(),
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
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
}
