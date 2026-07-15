#[cfg(test)]
mod test {
    use std::io::{Write, stdout};
    use std::{thread::sleep, time::Duration, vec};

    use secp256k1::PublicKey;

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
                sequence: 1,
            }],
            outputs: vec![TxOutput {
                value: 45,
                script_pub_key: client_locking_script.clone(),
            }],
            lock_time: Time::unix_timestamp(),
        };

        let signature = sign_tx(&tx.signing_hash(), &private_key);

        // sign transaction by miners private key because utxo belong to miner.
        let sign_script = Script {
            items: vec![
                ScriptItem::PushData(signature.serialize_der().to_vec()),
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
        side_chain_block.transactions[1].inputs[0].script_sig.items[0] = ScriptItem::PushData(
            sign_tx(
                &side_chain_block.transactions[1].signing_hash(),
                &private_key,
            )
            .serialize_der()
            .to_vec(),
        );

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

        let signature2 = sign_tx(
            &side_chain_tip.transactions[1].signing_hash(),
            &client_pvt_key2,
        );

        side_chain_tip.transactions[1].inputs[0].script_sig = Script {
            items: vec![
                ScriptItem::PushData(signature2.serialize_der().to_vec()),
                ScriptItem::PushData(client_pub_key2.serialize().to_vec()),
            ],
        };
        side_chain_tip.header.merkle_root =
            MerkleTree::compute_root(&side_chain_tip.transactions).unwrap();

        Miner::mine(&mut side_chain_tip).unwrap();

        println!("side tip block hash {:?}", side_chain_tip.header.hash());

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
        assert!(
            chain
                .ledger
                .get_utxo(&side_chain_block_outpoint)
                .is_some()
        ); 

        // verify every block still exists

        assert_eq!(chain.nodes.len(), 14); // then here it fails 
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
