use crate::{
    block::Block, difficulty::Difficulty, serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError, compact_size::{get_compact_size, read_compact_size}}, state_transition::StateTransition, types::{BigUint256, BlockHash},
};

#[derive(Clone, PartialEq, Eq)]
pub struct BlockNodeMetadata {
    pub hash: BlockHash,

    pub parent: Option<BlockHash>,

    pub height: u32,

    pub chain_work: BigUint256,

    pub state: Vec<StateTransition>,
}


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

impl BitcoinSerialize for BlockNodeMetadata {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(self.hash.as_bytes());

        if let Some(parent) = self.parent{
            bytes.extend(get_compact_size(32));
            bytes.extend_from_slice(parent.as_bytes());
        }else {
            bytes.extend(get_compact_size(1));
            bytes.push(0);
        };

        bytes.extend(self.height.to_le_bytes());
        bytes.extend_from_slice(self.chain_work.as_bytes());

        bytes.extend(get_compact_size(self.state.len()));
        for state in self.state.iter() {
            bytes.extend(state.serialize());
        };
        bytes
    }
}

impl BitcoinDeserialize for BlockNodeMetadata {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        let mut offset: usize = 0;

        let hash = BlockHash(bytes[offset..offset+32].try_into().map_err(|_| DeserializeError::InvalidCompactSize)?);
        offset += 32;
        let (compact_size, consumed) = read_compact_size(&bytes[offset..])?;
        offset += consumed;

        let parent = if compact_size == 1 {
            offset += 1;
            None
        } else {
            let parent =  BlockHash(bytes[offset..offset+32].try_into().map_err(|_| DeserializeError::InvalidCompactSize)?);
            offset += 32;
            Some(parent)
        };

        let height = u32::from_le_bytes(bytes[offset..offset+4].try_into().map_err(|_| DeserializeError::InvalidCompactSize)?);
        offset += 4;

        let chain_work = BigUint256(bytes[offset..offset+32].try_into().map_err(|_| DeserializeError::InvalidCompactSize)?);
        offset += 32;

        let (state_len, consumed) = read_compact_size(&bytes[offset..])?;
        offset += consumed;

        let mut state: Vec<StateTransition> = Vec::new();

        for _ in 0..state_len {
            let (state_transition, consumed) = StateTransition::deserialize(&bytes[offset..])?;
            offset += consumed;
            state.push(state_transition);
        };

        Ok((Self {
            hash,
            parent,
            height,
            chain_work,
            state
        },
         offset))

    }
}