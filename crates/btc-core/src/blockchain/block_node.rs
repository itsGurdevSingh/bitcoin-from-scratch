use crate::{
    block::Block, difficulty::Difficulty, state_transition::StateTransition, types::{BigUint256, BlockHash},
};

#[derive(Clone, PartialEq, Eq)]
pub struct BlockNode {
    pub block: Block,

    pub hash: BlockHash,

    pub parent: Option<BlockHash>,

    pub height: u32,

    pub chain_work: BigUint256,

    pub state: Vec<StateTransition>,
}

impl BlockNode {
    pub fn new(block: Block, state: Vec<StateTransition>, parent_node: Option<&BlockNode>) -> Self {
        let block_work = Difficulty::work(block.header.bits);

        match parent_node {
            // gensis block
            None => {
                return Self {
                    hash: block.header.hash(),
                    block,
                    state,
                    parent: None,
                    height: 0,
                    chain_work: block_work,
                };
            }
            //normal block
            Some(node) => {
                return Self {
                    hash: block.header.hash(),
                    block,
                    state,
                    parent: Some(node.hash.clone()),
                    height: node.height + 1,
                    chain_work: node.chain_work + block_work,
                };
            }
        };
    }
}
