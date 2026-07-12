use crate::{blockchain::{BlockNode, Blockchain}, types::BlockHash};

pub struct AncestorIter<'a> {
    pub blockchain: &'a Blockchain,
    pub current: Option<BlockHash>,
}

impl<'a> Iterator for AncestorIter<'a> {
    type Item = &'a BlockNode;

    fn next(&mut self) -> Option<Self::Item> {
        let hash = self.current?;

        let node = self.blockchain.nodes.get(&hash)?;

        self.current = node.parent;

        Some(node)
    }
}