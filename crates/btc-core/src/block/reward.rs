use crate::block::constants::{HALVING_INTERVAL, INITIAL_REWARD};

pub struct BlockReward;

impl BlockReward {
    pub fn total_reward(
        height: u32,
        fees: u64,
    ) -> u64 {
        BlockReward::subsidy(height) + fees
    }

    pub fn subsidy(height: u32) -> u64 {
    let halvings = height / HALVING_INTERVAL;
    INITIAL_REWARD >> halvings
}
}