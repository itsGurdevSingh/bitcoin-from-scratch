use std::collections::HashMap;

use crate::{
    block::{Block, BlockHeader, BlockReward},
    blockchain::{
        BlockNode, BlockProcessor, constants::INITIAL_BITS, error::BlockchainError,
        validator::ChainValidator,
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
    nodes: HashMap<BlockHash, BlockNode>,
    orphan_blocks: HashMap<BlockHash, Block>,
    tip: BlockHash,
    ledger: Ledger,
    mempool: Mempool,
}

impl Blockchain {
    pub fn new() -> Result<Self, BlockchainError> {
        let mut ledger = Ledger::new();
        let genesis = Self::create_genesis(&mut ledger)?;

        Ok(Self {
            tip: genesis.hash.clone(),
            nodes: HashMap::from([(genesis.hash.clone(), genesis)]),
            orphan_blocks: HashMap::new(),
            ledger,
            mempool: Mempool::new(),
        })
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        match self.nodes.get(&block.header.previous_block_hash) {
            Some(node) => {
                // validate
                ChainValidator::validate(&self, &block)?;

                let states = BlockProcessor::process(&block, &self.ledger)
                    .map_err(|e| BlockchainError::Processor(e))?;

                let node = BlockNode::new(block.clone(), states.clone(), Some(node));
                self.nodes.insert(block.header.hash(), node.clone());

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

                // check is any orphan is wating
                if let Some(child_block) = self.orphan_blocks.remove(&block.header.hash()) {
                    self.add_block(child_block)
                        .map_err(|_| BlockchainError::OrpanChildfailed)?;
                };

                // check is reorg needed
                if let Some(tip_node) = self.nodes.get(&self.tip) {
                    if node.chain_work > tip_node.chain_work {
                        // perform reorg .
                        // change tip.
                    }
                }
            }
            None => {
                self.orphan_blocks
                    .insert(block.header.previous_block_hash.clone(), block);
            }
        };
        Ok(())
    }

    pub fn median_timestamp(&self) -> Result<u64, BlockchainError> {
        let mut timestamps: Vec<u64> = Vec::new();

        let mut tip = self.tip_node()?;

        if tip.height == 0 {
            return Ok(tip.block.header.timestamp);
        }

        let start_height = if tip.height >= 11 { tip.height - 11 } else { 0 };

        while tip.height != start_height {
            timestamps.push(tip.block.header.timestamp);

            tip = self
                .nodes
                .get(&tip.parent.ok_or(BlockchainError::InvalidSyntex)?)
                .ok_or(BlockchainError::InvalidSyntex)?;
        }

        // if tip has height less then 11 then our loop not push fist blocks timstemp we have to push that.
        if start_height == 0 {
            timestamps.push(tip.block.header.timestamp);
        }

        // sort tiemstamps
        timestamps.sort();
        // is even
        if (timestamps.len() & 1) == 0 {
            let sec_idx = timestamps.len() / 2;
            return Ok((timestamps[sec_idx - 1] + timestamps[sec_idx]) / 2);
        }
        Ok(timestamps[timestamps.len() / 2])
    }

    pub fn tip_node(&self) -> Result<&BlockNode, BlockchainError> {
        self.nodes
            .get(&self.tip)
            .ok_or(BlockchainError::ChainIsEmpty)
    }

    pub fn height(&self) -> u32 {
        match self.nodes.get(&self.tip) {
            Some(node) => return node.height,
            None => return 0,
        };
    }

    pub fn expected_bits(&self) -> Result<u32, BlockchainError> {
        let tip = self.tip_node()?;

        if (tip.height + 1) % DIFFICULTY_WINDOW != 0 {
            return Ok(tip.block.header.bits);
        }

        let first_height = tip.height - (DIFFICULTY_WINDOW - 1);

        // find block with height on current node ;

        let first = self
            .get_node_by_height(tip, first_height)
            .ok_or(BlockchainError::InvalidSyntex)?
            .block
            .clone();

        let last = tip.block.clone();

        let actual_timespan = last.header.timestamp - first.header.timestamp;

        let bits = DifficultyAdjustment::next_bits(tip.block.header.bits, actual_timespan)
            .map_err(|e| BlockchainError::Difficulty(e))?;

        Ok(bits)
    }

    pub fn get_node_by_hash(&self, block_hash: BlockHash) -> Option<&BlockNode> {
        self.nodes.get(&block_hash)
    }
    pub fn get_node_by_height(&self, tip_node: &BlockNode, height: u32) -> Option<&BlockNode> {
        let mut tip = tip_node;

        if tip.height < height {
            return None;
        }
        loop {
            match tip.parent {
                Some(parent) => match self.nodes.get(&parent) {
                    Some(node) => {
                        if node.height == height {
                            return Some(node);
                        } else {
                            tip = node;
                        }
                    }
                    None => {}
                },
                None => {}
            }
        }
    }

    pub fn create_genesis(ledger: &mut Ledger) -> Result<BlockNode, BlockchainError> {
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

        let states =
            BlockProcessor::process(&block, ledger).map_err(|e| BlockchainError::Processor(e))?;

        // commit states to ledger
        for state in states.iter() {
            ledger
                .commit_state(state)
                .map_err(|e| BlockchainError::Ledger(e))?;
        }
        let _ = Miner::mine(&mut block);

        Ok(BlockNode::new(block, states, None))
    }

    // ledger
    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn ledger_mut(&mut self) -> &mut Ledger {
        &mut self.ledger
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{block::Builder, tests::dummy_tx::get_valid_tx};

    #[test]
    fn add_valid_block() {
        let mut chain = Blockchain::new().unwrap();
        let tx1 = get_valid_tx(&mut chain.ledger, 20, 2, 18);
        let tx2 = get_valid_tx(&mut chain.ledger, 10, 3, 9);

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

        let mut block = Builder::build(&[tx1, tx2], script, &chain).unwrap();

        block.header.timestamp += 1; // increment timstamp fo same time error of previous block
        Miner::mine(&mut block).unwrap();
        let block_hash = block.header.hash();

        chain.add_block(block).unwrap();
        // assert_eq!(chain.tip_node().unwrap().hash, block_hash)
        assert!(chain.nodes.contains_key(&block_hash))
    }
}
