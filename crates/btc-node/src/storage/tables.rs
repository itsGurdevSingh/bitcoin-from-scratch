use redb::TableDefinition;

pub const METADATA: TableDefinition<&str, &[u8]> = TableDefinition::new("metadata");

/// BLOCK HASH -> HEADER
pub const BLOCKS_HEADERS_TABLE: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("blocks_headers");

/// BLOCK HASH -> BLOCK
pub const BLOCKS_TABLE: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("blocks");

/// HEIGHT INDEXING (ONLY FOR ACTIVE CHAIN)
/// HEIGHT -> BLOCK HASH (FORM ACTIVE CHAIN)
pub const HEIGHT_INDEX_TABLE: TableDefinition<u32,&[u8; 32]> = TableDefinition::new("height_index");

/// BLOCK HASH -> BLOCK NODE METADATA
pub const BLOCK_NODE_METADATA_TABLE: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("block_node_metadata");


// for mempool 
/// TXID -> TRANSACTIONS
pub const TRANSACTIONS_TABLE: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("transactions_table");
/// TXID -> FEES
pub const TRANSACTIONS_FEES_TABLE: TableDefinition<&[u8; 32], u64> = TableDefinition::new("transaction_fees");


// utxo set LEDGER
/// OUTPOINT -> UTXOS (outpoint is consist of &[txid.. , vout] )
pub const UTXOS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("utxos");
