#[cfg(test)]
mod test {
    use std::io::{Write, stdout};
    use std::println;
use std::{thread::sleep, time::Duration, vec};

    use secp256k1::PublicKey;

    use crate::crypto::sha256d;
use crate::transaction::{TransactionSigHash, Witness};
    use crate::virtual_machine::SigHashType;
use crate::{
        block::Builder,
        blockchain::Blockchain,
        crypto::{generate_keypair_dummy, hash::hash160, sign_tx},
        merkle::MerkleTree,
        miner::Miner,
        script::{OpCode, Script, ScriptItem},
        tests::dummy_tx::get_valid_tx,
        transaction::{OutPoint, Transaction, TxInput, TxOutput},
        types::TxId,
        utils::time::Time,
    };

    #[test]
    fn add_valid_block() {
        let mut chain = Blockchain::new().unwrap();
        let tx1 = get_valid_tx(&mut chain.ledger, 20, 2, 18);
        let tx2 = get_valid_tx(&mut chain.ledger, 10, 3, 9);

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

        let mut block = Builder::build(&[tx1, tx2], script, &chain).unwrap();

        block.header.timestamp += 1; // increment timstamp fo same time error of previous block
        Miner::mine(&mut block).unwrap();
        let block_hash = block.header.hash();

        chain.add_block(block).unwrap();
        // assert_eq!(chain.tip_node().unwrap().hash, block_hash)
        assert!(chain.nodes.contains_key(&block_hash))
    }

    /// End-to-end integration test covering:
    ///
    /// 1. Linear chain growth.
    /// 2. Coinbase maturity.
    /// 3. Spending a matured coinbase output.
    /// 4. Creating a competing side chain.
    /// 5. Extending the side chain until it exceeds the active chain.
    /// 6. Automatic chain reorganization.
    ///
    /// We intentionally clone previously mined blocks to construct a side
    /// branch. The block builder always extends the current active tip,
    /// whereas this test needs to simulate a competing miner extending an
    /// earlier ancestor.
    #[test]
    fn chain_reorganization_flow() {
        // ---------------------------------------------------------------------
        // Setup blockchain and miner keys
        // ---------------------------------------------------------------------
        let (private_key, public_key) = generate_keypair_dummy();
        let mut chain = Blockchain::new().unwrap();
        // ---------------------------------------------------------------------
        // Mine enough blocks to satisfy coinbase maturity
        // ---------------------------------------------------------------------
        let miner_script = create_p2pkh_script(public_key);
        let mut first_cb_outpoint: OutPoint = OutPoint {
            txid: TxId([0u8; 32]),
            vout: 0,
        };

        for i in 0..10 {
            let mut block = Builder::build(&[], miner_script.clone(), &chain).unwrap();
            if i == 0 {
                block.header.timestamp += 1;
                first_cb_outpoint = OutPoint {
                    txid: block.transactions[0].txid(),
                    vout: 0,
                };
            }
            if i < 11 {
                // sleep the thread to save time colision.
                sleep(Duration::new(1, 0));
            };
            sleep(Duration::from_millis(25));

            Miner::mine(&mut block).unwrap();

            chain.add_block(block.clone()).unwrap();
            print!("\r block added {:2}", i + 1);
            stdout().flush().unwrap();
        }
        println!();

        // ---------------------------------------------------------------------
        // Spend the matured coinbase output on the active chain
        // ---------------------------------------------------------------------
        let (_client_pvt_key, client_pub_key) = generate_keypair_dummy();

        let client_locking_script = create_p2pkh_script(client_pub_key);

        let mut tx = Transaction {
            version: 1,
            inputs: vec![TxInput {
                previous_output: first_cb_outpoint,
                script_sig: Script { items: vec![] },
                witness: Witness::new(),
                sequence: 1,
            }],
            outputs: vec![TxOutput {
                value: 45,
                script_pub_key: client_locking_script.clone(),
            }],
            lock_time: Time::unix_timestamp(),
        };

        let utxo = chain.ledger.get_utxo(&tx.inputs[0].previous_output).unwrap().clone();
        let sig_hash = tx.signing_hash(0, &utxo.script_pub_key, SigHashType::All);

        let mut signature = sign_tx(&sig_hash, &private_key).serialize_der().to_vec();
        signature.extend((SigHashType::All as u32).to_le_bytes());

        // sign transaction by miners private key because utxo belong to miner.
        let sign_script = Script {
            items: vec![
                ScriptItem::PushData(signature),
                ScriptItem::PushData(public_key.serialize().to_vec()),
            ],
        };

        tx.inputs[0].script_sig = sign_script.clone();

        let mut active_chain_block = Builder::build(&[tx], miner_script.clone(), &chain).unwrap();

        Miner::mine(&mut active_chain_block).unwrap();

        chain.add_block(active_chain_block.clone()).unwrap();

        assert_eq!(chain.tip, active_chain_block.header.hash());

        // ---------------------------------------------------------------------
        // Create a competing block from the same parent
        // ---------------------------------------------------------------------

        let (client_pvt_key2, client_pub_key2) = generate_keypair_dummy();

        let client2_locking_script = create_p2pkh_script(client_pub_key2);

        let mut side_chain_block = active_chain_block.clone();
        side_chain_block.transactions[1].outputs[0].script_pub_key = client2_locking_script;

        let side_chain_sig_hash = side_chain_block.transactions[1].signing_hash(0,
             &utxo.script_pub_key,
              SigHashType::All);
        
        let mut signature_side_block = sign_tx(&side_chain_sig_hash, &private_key).serialize_der().to_vec();
        signature_side_block.extend((SigHashType::All as u32).to_le_bytes());

        side_chain_block.transactions[1].inputs[0].script_sig.items[0] = ScriptItem::PushData(signature_side_block);

        // update coinbase witness commitment.
        let mut witness_merkle_root = MerkleTree::compute_root_witness_v0(&side_chain_block.transactions[1..]).unwrap().into_bytes().to_vec();
        witness_merkle_root.extend([0u8; 32]);
        let witness_commitment = sha256d(&witness_merkle_root).to_vec();
        side_chain_block.transactions[0].outputs[1].script_pub_key.items[2] = ScriptItem::PushData(witness_commitment);

        let merkle = MerkleTree::compute_root(&side_chain_block.transactions).unwrap();
        side_chain_block.header.merkle_root = merkle;
        side_chain_block.header.nonce = 0;

        Miner::mine(&mut side_chain_block).unwrap();

        chain.add_block(side_chain_block.clone()).unwrap();

        assert_eq!(
            side_chain_block.header.hash(),
            chain
                .nodes
                .get(&side_chain_block.header.hash())
                .unwrap()
                .hash
        );

        // ---------------------------------------------------------------------
        // Extend the side chain
        // ---------------------------------------------------------------------
        let block_hash = side_chain_block.header.hash();
        let outpoint = OutPoint {
            txid: side_chain_block.transactions[1].txid(),
            vout: 0,
        };

        let mut side_chain_tip = side_chain_block.clone();

        side_chain_tip.transactions[1].inputs[0].previous_output = outpoint;
        side_chain_tip.transactions[1].outputs[0].value = 40;
        side_chain_tip.transactions[1].outputs[0]
            .script_pub_key
            .items[2] =
            ScriptItem::PushData(hash160(&client_pub_key2.serialize().to_vec()).to_vec());

        side_chain_tip.header.previous_block_hash = block_hash;

        let side_tip_unlocking_script = create_p2pkh_script(client_pub_key2);
        let side_chain_tip_sig_hash = side_chain_tip.transactions[1].signing_hash(0,
             &side_tip_unlocking_script,
              SigHashType::All);
        
        let mut signature_side_tip = sign_tx(&side_chain_tip_sig_hash, &client_pvt_key2).serialize_der().to_vec();
        signature_side_tip.extend((SigHashType::All as u32).to_le_bytes());

        side_chain_tip.transactions[1].inputs[0].script_sig = Script {
            items: vec![
                ScriptItem::PushData(signature_side_tip),
                ScriptItem::PushData(client_pub_key2.serialize().to_vec()),
            ],
        };

        
        // update coinbase witness commitment.
        let mut witness_merkle_root = MerkleTree::compute_root_witness_v0(&side_chain_tip.transactions[1..]).unwrap().into_bytes().to_vec();
        witness_merkle_root.extend([0u8; 32]);
        let witness_commitment = sha256d(&witness_merkle_root).to_vec();
        side_chain_tip.transactions[0].outputs[1].script_pub_key.items[2] = ScriptItem::PushData(witness_commitment);

        side_chain_tip.header.merkle_root =
            MerkleTree::compute_root(&side_chain_tip.transactions).unwrap();

        Miner::mine(&mut side_chain_tip).unwrap();

        chain.add_block(side_chain_tip.clone()).unwrap();

        assert_eq!(
            side_chain_tip.header.hash(),
            chain.nodes.get(&side_chain_tip.header.hash()).unwrap().hash
        );

        // ---------------------------------------------------------------------
        // Verify chain reorganization
        // ---------------------------------------------------------------------

        assert_eq!(chain.tip, side_chain_tip.header.hash());

        // verify ledger state
        let active_chain_block_outpoint = OutPoint {
            txid: active_chain_block.transactions[1].txid(),
            vout: 0,
        };

        let side_chain_block_outpoint = OutPoint {
            txid: side_chain_block.transactions[0].txid(),
            vout: 0,
        };
        // parent active chain now reorganized so its created utxo are not available in ledger.
        assert!(
            chain
                .ledger
                .get_utxo(&active_chain_block_outpoint)
                .is_none()
        );
        assert!(chain.ledger.get_utxo(&side_chain_block_outpoint).is_some());

        // verify every block still exists without depending on the exact node count
        assert!(chain.nodes.contains_key(&active_chain_block.header.hash()));
        assert!(chain.nodes.contains_key(&side_chain_block.header.hash()));
        assert!(chain.nodes.contains_key(&side_chain_tip.header.hash()));

        // ---------------------------------------------------------------------
        // Create an orphan parent/child pair from the side chain tip
        // ---------------------------------------------------------------------
        let (orphan_parent_pvt_key, orphan_parent_pub_key) = generate_keypair_dummy();
        let (_, orphan_parent_coinbase_pub_key) = generate_keypair_dummy();
        let (_, orphan_child_coinbase_pub_key) = generate_keypair_dummy();

        let orphan_parent_spend_outpoint = OutPoint {
            txid: side_chain_tip.transactions[1].txid(),
            vout: 0,
        };

        let mut orphan_parent_tx = Transaction {
            version: 1,
            inputs: vec![TxInput {
                previous_output: orphan_parent_spend_outpoint.clone(),
                script_sig: Script { items: vec![] },
                witness: Witness::new(),
                sequence: 1,
            }],
            outputs: vec![TxOutput {
                value: 35,
                script_pub_key: create_p2pkh_script(orphan_parent_pub_key),
            }],
            lock_time: Time::unix_timestamp(),
        };

        let orphan_parent_utxo = chain.ledger.get_utxo(&orphan_parent_spend_outpoint).unwrap();
        let orphan_block_sig_hash = orphan_parent_tx.signing_hash(0, &orphan_parent_utxo.script_pub_key, SigHashType::All);
        let mut orphan_parent_signature = sign_tx(&orphan_block_sig_hash, &client_pvt_key2).serialize_der().to_vec();
        orphan_parent_signature.extend((SigHashType::All as u32).to_le_bytes());
        orphan_parent_tx.inputs[0].script_sig = Script {
            items: vec![
                ScriptItem::PushData(orphan_parent_signature),
                ScriptItem::PushData(client_pub_key2.serialize().to_vec()),
            ],
        };

        let mut orphan_parent_block =
            Builder::build(&[orphan_parent_tx], miner_script.clone(), &chain).unwrap();
        orphan_parent_block.transactions[0].outputs[0].script_pub_key =
            create_p2pkh_script(orphan_parent_coinbase_pub_key);

            
        // update coinbase witness commitment.
        let mut witness_merkle_root = MerkleTree::compute_root_witness_v0(&orphan_parent_block.transactions[1..]).unwrap().into_bytes().to_vec();
        witness_merkle_root.extend([0u8; 32]);
        let witness_commitment = sha256d(&witness_merkle_root).to_vec();
        orphan_parent_block.transactions[0].outputs[1].script_pub_key.items[2] = ScriptItem::PushData(witness_commitment);


        orphan_parent_block.header.merkle_root =
            MerkleTree::compute_root(&orphan_parent_block.transactions).unwrap();
        orphan_parent_block.header.timestamp += 1;
        orphan_parent_block.header.nonce = 0;
        Miner::mine(&mut orphan_parent_block).unwrap();

        let orphan_parent_hash = orphan_parent_block.header.hash();
        let orphan_parent_created_outpoint = OutPoint {
            txid: orphan_parent_block.transactions[1].txid(),
            vout: 0,
        };

        let mut orphan_child_block = orphan_parent_block.clone();

        orphan_child_block.header.previous_block_hash = orphan_parent_hash;
        orphan_child_block.header.timestamp += 1;
        orphan_child_block.transactions[1].inputs[0].previous_output =
            orphan_parent_created_outpoint.clone();
        orphan_child_block.transactions[1].outputs[0].value = 30;
        orphan_child_block.transactions[1].outputs[0].script_pub_key =
            create_p2pkh_script(orphan_parent_pub_key);

        let mut orphan_child_tx = Transaction {
            version: 1,
            inputs: vec![TxInput {
                previous_output: orphan_parent_created_outpoint.clone(),
                script_sig: Script { items: vec![] },
                witness: Witness::new(),
                sequence: 1,
            }],
            outputs: vec![TxOutput {
                value: 30,
                script_pub_key: create_p2pkh_script(orphan_parent_pub_key),
            }],
            lock_time: Time::unix_timestamp(),
        };

        
        let unlocking_script = create_p2pkh_script(orphan_parent_pub_key);
        let orphan_child_sig_hash = orphan_child_tx.signing_hash(0, &unlocking_script, SigHashType::All);
        let mut signature_orphan_child = sign_tx(&orphan_child_sig_hash, &orphan_parent_pvt_key).serialize_der().to_vec();
        signature_orphan_child.extend((SigHashType::All as u32).to_le_bytes());

        orphan_child_tx.inputs[0].script_sig = Script {
            items: vec![
                ScriptItem::PushData(signature_orphan_child),
                ScriptItem::PushData(orphan_parent_pub_key.serialize().to_vec()),
            ],
        };
        orphan_child_block.transactions[0].outputs[0].script_pub_key =
            create_p2pkh_script(orphan_child_coinbase_pub_key);
        orphan_child_block.transactions[1] = orphan_child_tx;

        
        // update coinbase witness commitment.
        let mut witness_merkle_root = MerkleTree::compute_root_witness_v0(&orphan_child_block.transactions[1..]).unwrap().into_bytes().to_vec();
        witness_merkle_root.extend([0u8; 32]);
        let witness_commitment = sha256d(&witness_merkle_root).to_vec();
        orphan_child_block.transactions[0].outputs[1].script_pub_key.items[2] = ScriptItem::PushData(witness_commitment);

        orphan_child_block.header.merkle_root =
            MerkleTree::compute_root(&orphan_child_block.transactions).unwrap();
        orphan_child_block.header.nonce = 0;
        Miner::mine(&mut orphan_child_block).unwrap();

        let orphan_child_hash = orphan_child_block.header.hash();

        // send the child first so it becomes an orphan until the parent arrives
        chain.add_block(orphan_child_block.clone()).unwrap();
        assert!(chain.orphan_blocks.contains_key(&orphan_parent_hash));

        // now send the parent; the chain should process the waiting child too
        chain.add_block(orphan_parent_block.clone()).unwrap();

        assert!(chain.orphan_blocks.is_empty());
        assert!(chain.nodes.contains_key(&orphan_parent_hash));
        assert!(chain.nodes.contains_key(&orphan_child_hash));
        assert_eq!(chain.tip, orphan_child_hash);

        // the parent-created UTXO is spent by the child, and the child output is live
        assert!(
            chain
                .ledger
                .get_utxo(&orphan_parent_created_outpoint)
                .is_none()
        );
        assert!(
            chain
                .ledger
                .get_utxo(&OutPoint {
                    txid: orphan_child_block.transactions[1].txid(),
                    vout: 0,
                })
                .is_some()
        );
    }

    fn create_p2pkh_script(public_key: PublicKey) -> Script {
        let pub_key_hash = hash160(&public_key.serialize().to_vec());

        let p2pkh_script: Vec<ScriptItem> = vec![
            ScriptItem::Op(OpCode::Dup),
            ScriptItem::Op(OpCode::Hash160),
            ScriptItem::PushData(pub_key_hash.to_vec()),
            ScriptItem::Op(OpCode::EqualVerify),
            ScriptItem::Op(OpCode::CheckSig),
        ];

        Script {
            items: p2pkh_script,
        }
    }
}
