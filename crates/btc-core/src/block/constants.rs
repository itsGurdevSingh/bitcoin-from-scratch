use crate::virtual_machine::SigVersion;

pub const HALVING_INTERVAL: u32 = 210000;
pub const INITIAL_REWARD: u64 = 50;
pub const MAX_BLOCK_SIZE: usize = 1000000;
pub const MAX_BLOCK_WEIGHT: usize = 4000000;
pub const MAX_BLOCK_SIG_OP_COST: u32 = 80_000;

pub const SIG_VERSION: SigVersion = SigVersion::WitnessV0;
pub const WITNESS_COMMITMENT_HEADER: [u8; 4] = [
    0xaa,
    0x21,
    0xa9,
    0xed,
];