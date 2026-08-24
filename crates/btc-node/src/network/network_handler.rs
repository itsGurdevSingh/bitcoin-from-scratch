use std::sync::Arc;

use btc_core::{block::Block, serialization::BitcoinDeserialize, transaction::Transaction};
use tokio::sync::RwLock;

use crate::{
    network::{NetworkError, peer::PeerId},
    node::Node,
};

pub struct NetworkHandler {
    node: Arc<RwLock<Node>>,
    peer_id: PeerId,
}

impl NetworkHandler {
    pub fn new(
        node: Arc<RwLock<Node>>,
        peer_id: PeerId,
    ) -> Self {
        Self {
            node,
            peer_id,
        }
    }

    pub async fn handle_transaction(&self, tx_bytes: Vec<u8>) -> Result<(), NetworkError> {
        let (tx, _) = Transaction::deserialize(&tx_bytes).map_err(NetworkError::Deserialize)?;
        self.node
            .write()
            .await
            .submit_transaction(tx, Some(self.peer_id))
            .await
            .map_err(NetworkError::Node)
    }

    pub async fn handle_block(&self, block_bytes: Vec<u8>) -> Result<(), NetworkError> {
        let (block, _) = Block::deserialize(&block_bytes).map_err(NetworkError::Deserialize)?;
        self.node
            .write()
            .await
            .submit_block(block, Some(self.peer_id))
            .await
            .map_err(NetworkError::Node)
    }
}
