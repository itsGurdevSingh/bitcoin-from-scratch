use std::collections::{HashMap, HashSet};

use crate::{
    blockchain::{BlockNode, Blockchain},
    ledger::Ledger,
    transaction::OutPoint,
    types::BlockHash,
    utxo::Utxo,
};

pub struct Overlay {
    pub unspent_utxos: HashMap<OutPoint, Utxo>,
    pub spent_utxos: HashSet<OutPoint>,
}

impl Overlay {
    pub fn new(chain: &Blockchain, current_tip: &BlockNode, partent_node: &BlockNode) -> Self {
        let mut overlay = Self {
            unspent_utxos: HashMap::new(),
            spent_utxos: HashSet::new(),
        };
        if current_tip == partent_node {
            return overlay;
        }
        overlay.create_overlay(chain, current_tip, partent_node);
        overlay
    }

    pub fn lookup<'a>(&'a self, ledger: &'a Ledger, outpoint: &'a OutPoint) -> Option<&'a Utxo> {
        if let Some(utxo) = self.unspent_utxos.get(outpoint) {
            return Some(utxo);
        };

        if let Some(_) = self.spent_utxos.get(outpoint) {
            return None;
        };

        ledger.get_utxo(outpoint)
    }

    pub fn create_overlay(
        &mut self,
        chain: &Blockchain,
        current_tip: &BlockNode,
        partent_node: &BlockNode,
    ) {
        let common_ancestor = chain.find_common_ancestor(current_tip, partent_node).unwrap();

        self.tip_branch(chain, current_tip, &common_ancestor);
        self.parent_branch(chain, &common_ancestor, partent_node);
    }

    pub fn tip_branch(&mut self, chain: &Blockchain, old_tip: &BlockNode, ancestor: &BlockNode) {
        let mut path: Vec<BlockHash> = Vec::new();
        for node in chain.ancestors(old_tip.hash) {
            if node.hash == ancestor.hash {
                break;
            }
            path.push(node.hash);
        }

        for hash in path {
            match chain.nodes.get(&hash) {
                Some(node) => {
                    // remove form ledger and get fees map
                    for state in node.state.iter() {
                        for cu in state.created_utxos.iter() {
                            self.unspent_utxos.remove(&cu.outpoint);
                            self.spent_utxos.insert(cu.outpoint.clone());
                        }

                        for sp in state.spent_utxos.iter() {
                            self.spent_utxos.remove(&sp.outpoint);
                            self.unspent_utxos
                                .insert(sp.outpoint.clone(), sp.utxo.clone());
                        }
                    }
                }
                None => {}
            };
        }
    }

    pub fn parent_branch(&mut self, chain: &Blockchain, ancestor: &BlockNode, partent_node: &BlockNode) {
        let mut path: Vec<BlockHash> = Vec::new();
        for node in chain.ancestors(partent_node.hash) {
            if node.hash == ancestor.hash {
                break;
            }
            path.push(node.hash);
        }

        path.reverse();

        for hash in path {
            match chain.nodes.get(&hash) {
                Some(node) => {
                    for state in node.state.iter() {
                        for cu in state.created_utxos.iter() {
                            self.spent_utxos.remove(&cu.outpoint);
                            self.unspent_utxos
                                .insert(cu.outpoint.clone(), cu.utxo.clone());
                        }

                        for sp in state.spent_utxos.iter() {
                            self.unspent_utxos.remove(&sp.outpoint);
                            self.spent_utxos.insert(sp.outpoint.clone());
                        }
                    }
                }
                None => {}
            };
        }
    }
}
