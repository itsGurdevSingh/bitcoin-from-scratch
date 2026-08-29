use std::{collections::HashSet, sync::Arc};

use btc_core::{
    block::{Block, BlockHeader},
    serialization::BitcoinDeserialize,
    transaction::Transaction,
    types::{BlockHash, TxId},
};
use tokio::sync::RwLock;

use crate::{
    network::{
        GetDataMessage, GetHeadersMessage, HeadersMessage, InvMessage, InventoryType,
        InventoryVector, NetworkError, inventory_manager::InventoryState, peer::PeerId,
    },
    node::Node,
};

pub struct NetworkHandler {
    node: Arc<RwLock<Node>>,
    peer_id: PeerId,
}

impl NetworkHandler {
    pub fn new(node: Arc<RwLock<Node>>, peer_id: PeerId) -> Self {
        Self { node, peer_id }
    }

    pub async fn handle_transaction(&self, tx_bytes: Vec<u8>) -> Result<(), NetworkError> {
        let (tx, _) = Transaction::deserialize(&tx_bytes).map_err(NetworkError::Deserialize)?;

        let mut n = self.node.write().await;
        let inv_vec = InventoryVector::new(InventoryType::Block, tx.txid().into_bytes());
        let mut origin_peers = HashSet::new();
        {
            let mut inv = n.inventory.write().await;
            // check are we asked for that data.
            if let Some(entry) = inv.get_entry(&inv_vec) {
                if entry.state == InventoryState::Requested {
                    origin_peers = entry.announced_by.clone();
                    inv.mark_processing(&inv_vec);
                }
            } else {
                Err(NetworkError::DataNotRequested)?;
            };
        }

        n.submit_transaction(tx, origin_peers)
            .await
            .map_err(NetworkError::Node)?;

        let mut inv = n.inventory.write().await;
        inv.remove_entry(&inv_vec);

        Ok(())
    }

    pub async fn handle_block(&self, block_bytes: Vec<u8>) -> Result<(), NetworkError> {
        let (block, _) = Block::deserialize(&block_bytes).map_err(NetworkError::Deserialize)?;
        let mut n = self.node.write().await;
        let inv_vec = InventoryVector::new(InventoryType::Block, block.header.hash().into_bytes());
        let mut origin_peers = HashSet::new();
        {
            let mut inv = n.inventory.write().await;
            // check are we asked for that data.
            if let Some(entry) = inv.get_entry(&inv_vec) {
                if entry.state == InventoryState::Requested {
                    origin_peers = entry.announced_by.clone();
                    inv.mark_processing(&inv_vec);
                }
            } else {
                Err(NetworkError::DataNotRequested)?;
            };
        }

        n.submit_block(block, origin_peers)
            .await
            .map_err(NetworkError::Node)?;

        let mut inv = n.inventory.write().await;
        inv.remove_entry(&inv_vec);

        Ok(())
    }

    pub async fn mark_requesing_data(&self, inv_vec: InventoryVector) {
        let n = self.node.read().await;
        let mut inv_manager = n.inventory.write().await;
        inv_manager.mark_requested(&inv_vec, self.peer_id);
    }

    pub async fn check_inventory(&self, inventory_bytes: Vec<u8>) -> GetDataMessage {
        let (inv_msg, _) = InvMessage::deserialize(&inventory_bytes).unwrap();

        let mut get_data_inv: Vec<InventoryVector> = Vec::new();

        for inv_vector in inv_msg.inventory {
            let n = self.node.read().await;
            let mut inv_manager = n.inventory.write().await;
            match inv_vector.inv_type {
                InventoryType::Block => {
                    if !n.has_block(&BlockHash(inv_vector.hash)).await {
                        // we use else approce to save unnessary clone.
                        if !inv_manager.has_entry(&inv_vector) {
                            // if we not has entry then we create new and also return getdata msg.
                            inv_manager.add_announcer(&inv_vector, self.peer_id); // create new entry or add announcer.
                            get_data_inv.push(inv_vector);
                        } else {
                            inv_manager.add_announcer(&inv_vector, self.peer_id); // create new entry or add announcer.
                        }
                    }
                }

                // Todo impl when block part is done.
                InventoryType::Tx => {
                    if !n.has_transaction(&TxId(inv_vector.hash)).await {
                        if !inv_manager.has_entry(&inv_vector) {
                            // if we not has entry then we create new and also return getdata msg.
                            inv_manager.add_announcer(&inv_vector, self.peer_id); // create new entry or add announcer.
                            get_data_inv.push(inv_vector);
                        } else {
                            inv_manager.add_announcer(&inv_vector, self.peer_id); // create new entry or add announcer.
                        }
                    }
                }
            }
        }

        GetDataMessage {
            inventory: get_data_inv,
        }
    }

    pub async fn get_block(&self, block_hash: [u8; 32]) -> Result<Block, NetworkError> {
        let n = self.node.read().await;
        if let Some(node) = n.chain.get_node_by_hash(BlockHash(block_hash)) {
            Ok(node.block)
        } else {
            Err(NetworkError::DataNotAnnounced)
        }
    }
    pub async fn get_tx(&self, txid: [u8; 32]) -> Result<Transaction, NetworkError> {
        let n = self.node.read().await;

        if let Some(entry) = n.chain.mempool.get_transaction(&TxId(txid)) {
            Ok(entry.tx.clone())
        } else {
            Err(NetworkError::DataNotAnnounced)
        }
    }

    pub async fn headers_for_locators(&self, locators: Vec<BlockHash>) -> Vec<BlockHeader> {
        let n = self.node.read().await;
        n.headers_for_locators(locators)
    }

    pub async fn build_locator(&self) -> Result<Vec<BlockHash>, NetworkError> {
        let n = self.node.read().await;
        n.build_locator().map_err(NetworkError::Node)
    }

    pub async fn handle_headers(
        &self,
        headers_bytes: Vec<u8>,
    ) -> Result<Vec<BlockHash>, NetworkError> {
        let (headers_message, _) = HeadersMessage::deserialize(&headers_bytes)
            .map_err(|_| NetworkError::TypeCastFailed)?;
        let n = self.node.read().await;

        let mut verified_hashes: Vec<BlockHash> = Vec::new();

        for header in headers_message.headers {
            if n.chain.verify_header(&header) {
                verified_hashes.push(header.hash());
            } else {
                break;
            }
        }

        Ok(verified_hashes)
    }

    pub async fn handle_get_headers(
        &self,
        message_bytes: Vec<u8>,
    ) -> Result<HeadersMessage, NetworkError> {
        let (get_headers_message, _) = GetHeadersMessage::deserialize(&message_bytes)
            .map_err(|_| NetworkError::TypeCastFailed)?;

        let headers = HeadersMessage::new(
            self.headers_for_locators(get_headers_message.locator_hashes)
                .await,
        );
        Ok(headers)
    }
}
