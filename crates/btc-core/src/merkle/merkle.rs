use crate::{
    crypto::sha256d,
    merkle::MerkleError,
    transaction::Transaction,
    types::{MerkleRoot, TxId},
};

pub struct MerkleTree;

pub struct MerkleProof {
    pub proof: Vec<MerkleNode>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub position: NodePosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum NodePosition {
    Left,
    Right,
}

impl MerkleProof {
    fn new() -> Self {
        Self { proof: Vec::new() }
    }
}

impl MerkleTree {
    pub fn compute_root(transactions: &[Transaction]) -> Result<MerkleRoot, MerkleError> {
        if transactions.is_empty() {
            return Err(MerkleError::EmptyTransactionList);
        }

        let mut current_level = Vec::new();

        for tx in transactions.iter() {
            let hash = tx.txid().into_bytes();
            current_level.push(hash);
        }

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            let mut idx: usize = 0;

            while current_level.len() > idx {
                let (left_hash, right_hash) = match current_level.len() > idx + 1 {
                    true => (&current_level[idx], &current_level[idx + 1]),
                    false => (&current_level[idx], &current_level[idx]),
                };

                let merkle_hash = Self::hash_markle_pair(left_hash, right_hash);
                next_level.push(merkle_hash);
                idx += 2;
            }
            current_level = next_level;
        }
        Ok(MerkleRoot(current_level[0]))
    }

    pub fn build_proof(
        transactions: &[Transaction],
        txid: TxId,
    ) -> Result<MerkleProof, MerkleError> {
        if transactions.is_empty() {
            return Err(MerkleError::EmptyTransactionList);
        }

        let mut merkle_proof = MerkleProof::new();
        let mut target = txid.into_bytes();
        let mut current_level = Vec::new();

        {
            let mut has_target = false;
            for tx in transactions.iter() {
                let hash = tx.txid().into_bytes();
                current_level.push(hash);
                if hash == target {
                    has_target = true;
                }
            }

            if !has_target {
                return Err(MerkleError::TransactionNotFound);
            }
        }

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            let mut idx: usize = 0;

            while current_level.len() > idx {
                let (left_hash, right_hash) = match current_level.len() > idx + 1 {
                    true => (&current_level[idx], &current_level[idx + 1]),
                    false => (&current_level[idx], &current_level[idx]),
                };

                let merkle_hash = Self::hash_markle_pair(left_hash, right_hash);

                if *left_hash == target {
                    merkle_proof.proof.push(MerkleNode {
                        hash: *right_hash,
                        position: NodePosition::Right,
                    });
                    target = merkle_hash;
                } else if *right_hash == target {
                    merkle_proof.proof.push(MerkleNode {
                        hash: *left_hash,
                        position: NodePosition::Left,
                    });
                    target = merkle_hash;
                }

                next_level.push(merkle_hash);

                idx += 2;
            }

            current_level = next_level;
        }

        Ok(merkle_proof)
    }

    pub fn verify_proof(txid: TxId, merkle_proof: &MerkleProof, root: &MerkleRoot) -> Result<(),MerkleError> {

        let mut current = txid.into_bytes();

        for node in merkle_proof.proof.iter() {
           current =  match node.position {
                NodePosition::Left => Self::hash_markle_pair(&node.hash, &current),
                NodePosition::Right => Self::hash_markle_pair(&current, &node.hash)
            };
        }

        if current != root.into_bytes() {
            return Err(MerkleError::InvalidProof);
        };

        Ok(())
    }

    fn hash_markle_pair(left_hash: &[u8; 32], right_hash: &[u8; 32]) -> [u8; 32] {
        let mut data = Vec::with_capacity(64);

        data.extend_from_slice(left_hash);
        data.extend_from_slice(right_hash);

        sha256d(&data)
    }
}
