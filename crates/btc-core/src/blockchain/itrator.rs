use crate::{blockchain::{BlockNode, Blockchain}, presistaence::DbPersistence, types::BlockHash};

pub struct AncestorIter<'a, S: DbPersistence> {
    pub blockchain: &'a Blockchain<S>,
    pub current: Option<BlockHash>,
}

impl<'a, S: DbPersistence> Iterator for AncestorIter<'a, S> {
    type Item = BlockNode;

    fn next(&mut self) -> Option<Self::Item> {
        let hash = self.current?;

        let node = self.blockchain.nodes.get(&hash)?;

        self.current = node.parent;

        Some(node)
    }
}