pub const MAGIC:[u8; 4] = [1,2,3,4];

pub const PROTOCOL_VERSION: i32 = 70015;
pub const MIN_PEER_PROTOCOL_VERSION: i32 = 70015;

pub const NODE_NETWORK: u64 = 1 << 0;
pub const ALLOWED_SERVICES: [u64; 1] = [NODE_NETWORK];  // later we will add more service then we also increae arry size.

pub const USER_AGENT: &'static str = "BTC-Core";