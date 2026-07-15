use std::collections::HashMap;

use crate::{
    blockchain::{BlockNode, Blockchain, error::BlockchainError}, types::{BlockHash, TxId},
};

impl Blockchain {
    pub fn reorg(&mut self, new_tip_node: &BlockNode) -> Result<(), BlockchainError>{

        let current_tip_node = self.tip_node()?.clone();
        let common_ancestor = self.find_common_ancestor(&current_tip_node, new_tip_node).unwrap();
        
        self.disconnect_path(&current_tip_node, &common_ancestor);
        self.connect_path(&common_ancestor, new_tip_node);

        Ok(())
    }

    fn disconnect_path(&mut self, old_tip: &BlockNode, ancestor: &BlockNode) {
        let mut path: Vec<BlockHash> = Vec::new();
        for node in self.ancestors(old_tip.hash) {
            if node.hash == ancestor.hash {
                break;
            }
            path.push(node.hash);
        }

        for hash in path {
            let node = self.nodes[&hash].clone();
            let mut fees_map: HashMap<TxId, u64> = HashMap::new();
            // remove form ledger and get fees map 
            for state in node.state.iter() {
                fees_map.insert(state.created_utxos[0].outpoint.txid, state.fee);
                let _ = self.ledger.rollback_state(state);
            }

            // add back to mempool
            for tx in node.block.transactions {
                match fees_map.get(&tx.txid()) {
                    Some(fee) => {
                        let _ = self.mempool.add_transaction(tx, *fee);
                    }
                    None => {}
                };
            }
        }
    }

    fn connect_path(&mut self, ancestor: &BlockNode, new_tip: &BlockNode) {
        let mut path: Vec<BlockHash> = Vec::new();
        for node in self.ancestors(new_tip.hash) {
            if node.hash == ancestor.hash {
                break;
            }
            path.push(node.hash);
        }

        path.reverse();

        for hash in path {
            let node = self.nodes[&hash].clone();

            // comit to ledger 
            for state in node.state.iter() {
                let _ = self.ledger.commit_state(state);
            }

            // remove form mempool
            for tx in node.block.transactions.iter() {
                self.mempool.remove_transaction(&tx.txid());
            }
        }
    }
}
