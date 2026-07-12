use std::collections::HashMap;

use crate::{
    blockchain::{BlockNode, Blockchain},
    types::{BlockHash, TxId},
};

impl Blockchain {
    pub fn reorg(&mut self, current_tip: &BlockNode, new_tip: &BlockNode) {
        let common_ancestor = self.find_common_ancestor(current_tip, new_tip).unwrap();

        self.disconnect_path(current_tip, &common_ancestor);
        self.connect_path(&common_ancestor, new_tip);
    }

    pub fn disconnect_path(&mut self, old_tip: &BlockNode, ancestor: &BlockNode) {
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

    pub fn connect_path(&mut self, ancestor: &BlockNode, new_tip: &BlockNode) {
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
