use crate::{
    block::{Block, BlockHeader, BlockReward},
    blockchain::{
        BlockProcessor, constants::INITIAL_BITS, error::BlockchainError, validator::ChainValidator,
    },
    difficulty::{DifficultyAdjustment, constants::DIFFICULTY_WINDOW},
    ledger::Ledger,
    mempool::Mempool,
    miner::Miner,
    script::{OpCode, Script, ScriptItem},
    transaction::CoinBase,
    types::{BlockHash, MerkleRoot},
    utils::time::Time,
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
        // validate
        ChainValidator::validate(&self, &block)?;

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
            // ignore mempool error because its not nessary we have all tx in mempool if block is proposed by other miner.
            self.mempool.remove_transaction(&tx.txid());
        }
        self.blocks.push(block);

        Ok(())
    }

    pub fn median_timestamp(&self) -> u64 {
        let mut timestamps: Vec<u64> = Vec::new();

        let start = self.blocks.len().saturating_sub(11).max(0);
        for block in &self.blocks[start..] {
            timestamps.push(block.header.timestamp);
        }
        // sort tiemstamps
        timestamps.sort();
        // is even
        if (timestamps.len() & 1) == 0 {
            let sec_idx = timestamps.len() / 2;
            return (timestamps[sec_idx - 1] + timestamps[sec_idx]) / 2;
        }
        timestamps[timestamps.len() / 2]
    }

    pub fn tip(&self) -> Result<&Block, BlockchainError> {
        let block = self.blocks.last().ok_or(BlockchainError::ChainIsEmpty)?;
        Ok(block)
    }

    pub fn height(&self) -> u32 {
        self.blocks.len() as u32
    }

    pub fn expected_bits(&self) -> Result<u32, BlockchainError> {
        let tip = self.tip()?;

        let next_height = self.height() + 1;
        if next_height % DIFFICULTY_WINDOW != 0 {
            return Ok(tip.header.bits);
        }

        let first = &self.blocks[(next_height - DIFFICULTY_WINDOW) as usize];
        let last = tip;

        let actual_timespan = last.header.timestamp - first.header.timestamp;

        let bits = DifficultyAdjustment::next_bits(tip.header.bits, actual_timespan)
            .map_err(|e| BlockchainError::Difficulty(e))?;

        Ok(bits)
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
                version: 1,
                previous_block_hash: BlockHash([0u8; 32]),
                merkle_root: MerkleRoot(transaction.txid().into_bytes()),
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
