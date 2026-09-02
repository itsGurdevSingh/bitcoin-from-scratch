use btc_core::{blockchain::{config::GenesisConfig, constants::INITIAL_BITS}, types::MerkleRoot};

pub const MAGIC:[u8; 4] = [1,2,3,4];

pub const PROTOCOL_VERSION: i32 = 70015;
pub const MIN_PEER_PROTOCOL_VERSION: i32 = 70015;

pub const NODE_NETWORK: u64 = 1 << 0;
pub const ALLOWED_SERVICES: [u64; 1] = [NODE_NETWORK];  // later we will add more service then we also increae arry size.

pub const USER_AGENT: &'static str = "BTC-Core";

pub const HEADERS_MAX_BATCH_SIZE: usize = 2000;

pub const GENSIS_CONFIG: GenesisConfig = GenesisConfig {
            timestamp: 1788281890,
            nonce: 38546,
            bits: INITIAL_BITS,
            merkle_root: MerkleRoot([
                22, 60, 108, 82, 117, 38, 74, 123, 142, 232, 183, 65, 71, 204, 27, 17, 155, 55,
                126, 82, 38, 227, 92, 29, 159, 163, 196, 75, 230, 40, 30, 73,
            ]),
        };