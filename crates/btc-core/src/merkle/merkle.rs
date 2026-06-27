use crate::{
    crypto::sha256d,
    merkle::MerkleError,
    transaction::Transaction,
    types::{MerkleRoot, TxId},
};

#[derive(Debug)]
pub struct MerkleTree;

#[derive(PartialEq, Eq, Debug)]
pub struct MerkleProof {
    pub proof: Vec<MerkleNode>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub position: NodePosition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodePosition {
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

    pub fn verify_proof(
        txid: TxId,
        merkle_proof: &MerkleProof,
        root: &MerkleRoot,
    ) -> Result<(), MerkleError> {
        let mut current = txid.into_bytes();

        for node in merkle_proof.proof.iter() {
            current = match node.position {
                NodePosition::Left => Self::hash_markle_pair(&node.hash, &current),
                NodePosition::Right => Self::hash_markle_pair(&current, &node.hash),
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

#[cfg(test)]
mod test {

    use crate::{
        crypto::{generate_keypair_dummy, hash::hash160, sign_tx}, ledger::Ledger, script::{OpCode, Script, ScriptItem}, transaction::{self, OutPoint, TxInput, TxOutput}, utxo::Utxo,
    };

    use super::*;

    //     compute_root
    // -------------
    // ✓ empty list
    #[test]
    fn empty_transaction_list() {
        assert_eq!(
            MerkleTree::compute_root(&[]),
            Err(MerkleError::EmptyTransactionList)
        )
    }
    // ✓ one tx
    #[test]
    fn single_transaction_root() {
        let (tx, _) = get_valid_tx(2);

        assert_eq!(
            MerkleTree::compute_root(&[tx.clone()]),
            Ok(MerkleRoot(tx.txid().into_bytes()))
        )
    }

    // ✓ two tx
    #[test]
    fn two_transaction_root() {
        let (tx, _) = get_valid_tx(2);
        let (tx2, _) = get_valid_tx(4);

        let mut data = Vec::with_capacity(64);

        data.extend_from_slice(&tx.txid().into_bytes());
        data.extend_from_slice(&tx2.txid().into_bytes());

        let root = sha256d(&data);

        assert_eq!(
            MerkleTree::compute_root(&[tx.clone(), tx2]),
            Ok(MerkleRoot(root))
        )
    }

    // ✓ odd tx count
    #[test]
    fn three_transaction_root_duplicates_last() {
        let (tx1, _) = get_valid_tx(2);
        let (tx2, _) = get_valid_tx(4);
        let (tx3, _) = get_valid_tx(5);

        // let root = sha256d(&data);

        let transaction = [tx1.clone(), tx2.clone(), tx3.clone()];

        let data1 = MerkleTree::hash_markle_pair(tx1.txid().as_bytes(), tx2.txid().as_bytes());
        let data2 = MerkleTree::hash_markle_pair(tx3.txid().as_bytes(), tx3.txid().as_bytes());
        let root = MerkleTree::hash_markle_pair(&data1, &data2);
        assert_eq!(MerkleTree::compute_root(&transaction), Ok(MerkleRoot(root)))
    }
    // ✓ deterministic
    #[test]
    fn same_transactions_same_root() {
        let (tx, _) = get_valid_tx(2);

        assert_eq!(
            MerkleTree::compute_root(&[tx.clone()]),
            MerkleTree::compute_root(&[tx.clone()])
        )
    }

    // build_proof
    // -------------
    // ✓ tx not found
    #[test]
    fn transaction_not_found() {
        let (tx1, _) = get_valid_tx(2);
        let (tx2, _) = get_valid_tx(4);
        let (tx3, _) = get_valid_tx(5);

        assert_eq!(
            MerkleTree::build_proof(&[tx1, tx2], tx3.txid()),
            Err(MerkleError::TransactionNotFound)
        )
    }
    // ✓ one tx
    #[test]
    fn single_transaction_proof() {
        let (tx, _) = get_valid_tx(2);

        let res = MerkleTree::build_proof(&[tx.clone()], tx.txid()).unwrap();
        assert!(res.proof.is_empty())
    }

    // ✓ valid proof
    #[test]
    fn build_valid_proof() {
        let (tx1, _) = get_valid_tx(2);
        let (tx2, _) = get_valid_tx(4);
        let (tx3, _) = get_valid_tx(5);
        let (tx4, _) = get_valid_tx(6);

        let res = MerkleTree::build_proof(&[tx1.clone(), tx2, tx3, tx4], tx1.txid()).unwrap();

        assert!(res.proof.len() == 2)
    }

    // verify_proof
    // -------------
    // ✓ valid proof
    #[test]
    fn verify_valid_proof() {
        let (tx1, _) = get_valid_tx(2);
        let (tx2, _) = get_valid_tx(4);
        let (tx3, _) = get_valid_tx(5);
        let (tx4, _) = get_valid_tx(6);

        let transaction = [tx1.clone(), tx2, tx3, tx4];

        let root = MerkleTree::compute_root(&transaction).unwrap();

        let merkle_proof = MerkleTree::build_proof(&transaction.clone(), tx1.txid()).unwrap();

        let res = MerkleTree::verify_proof(tx1.txid(), &merkle_proof, &root);

        assert!(res.is_ok())
    }
    // ✓ modified proof
    #[test]
    fn modified_proof_fails() {
        let (tx1, _) = get_valid_tx(2);
        let (tx2, _) = get_valid_tx(4);
        let (tx3, _) = get_valid_tx(5);
        let (tx4, _) = get_valid_tx(6);

        let transaction = [tx1.clone(), tx2.clone(), tx3, tx4];

        let root = MerkleTree::compute_root(&transaction).unwrap();

        let merkle_proof = MerkleTree::build_proof(&transaction.clone(), tx1.txid()).unwrap();

        let res = MerkleTree::verify_proof(tx2.txid(), &merkle_proof, &root);

        assert_eq!(res, Err(MerkleError::InvalidProof))
    }
    // ✓ wrong root
    #[test]
    fn wrong_root_fails() {
        let (tx1, _) = get_valid_tx(2);
        let (tx2, _) = get_valid_tx(4);
        let (tx3, _) = get_valid_tx(5);
        let (tx4, _) = get_valid_tx(6);

        let transaction = [tx1.clone(), tx2, tx3, tx4];

        let root = MerkleRoot([0u8; 32]);

        let merkle_proof = MerkleTree::build_proof(&transaction.clone(), tx1.txid()).unwrap();

        let res = MerkleTree::verify_proof(tx1.txid(), &merkle_proof, &root);

        assert_eq!(res, Err(MerkleError::InvalidProof))
    }

    fn get_valid_tx(val: u64) -> (Transaction, Ledger) {
        let tx_input = create_dummy_tx_input();
        let tx_output = create_dummy_tx_output(val);

        // for adding utxo for making input valid and for geting utxo for that input for pub_key_script .
        let mut ledger = Ledger::new();

        let mut transaction = Transaction {
            version: 10,
            inputs: vec![tx_input],
            outputs: vec![tx_output],
            lock_time: 1000,
        };

        // get message serilize transaction and double hash that.
        // let serialize = transaction.serialize();
        let message = transaction.signing_hash();

        for input in transaction.inputs.iter_mut() {
            // that's wallets responsibility how it handles key for testing we use dummy keys .
            let (sk, pk) = generate_keypair_dummy();

            let sig = sign_tx(&message, &sk).serialize_der().to_vec();

            let script = Script {
                items: vec![
                    ScriptItem::PushData(sig),                     // signature
                    ScriptItem::PushData(pk.serialize().to_vec()), // public key
                ],
            };
            input.script_sig = script;

            // add valid utxo
            let utxo = create_dummy_utxo(10, hash160(&pk.serialize().to_vec()).to_vec());

            let _ = ledger.add_utxo(input.previous_output.clone(), utxo);
        }

        (transaction, ledger)
    }

    fn create_dummy_tx_input() -> TxInput {
        let sig_script_items: Vec<ScriptItem> = vec![
            ScriptItem::PushData(vec![0u8; 32]),
            ScriptItem::PushData(vec![0u8; 64]),
        ];

        let script_sig = Script {
            items: sig_script_items,
        };

        let previous_output = OutPoint {
            txid: TxId([0u8; 32]),
            vout: 8,
        };

        TxInput {
            previous_output,
            script_sig,
            sequence: 5,
        }
    }

    fn create_dummy_tx_output(val: u64) -> TxOutput {
        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(vec![0u8; 20]), // 20-byte dummy pubkey hash
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        let script: Script = Script {
            items: p2pkh_script,
        };

        TxOutput {
            value: val,
            script_pub_key: script,
        }
    }

    fn create_dummy_utxo(val: u64, pkh: Vec<u8>) -> Utxo {
        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(pkh),
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        Utxo {
            value: val,
            script_pub_key: Script {
                items: p2pkh_script,
            },
            is_coinbase: false,
            block_height: 1000,
        }
    }
}
