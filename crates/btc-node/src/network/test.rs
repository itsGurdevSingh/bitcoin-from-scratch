#[cfg(test)]
mod tests {
    use btc_core::{
        block::{Block, Builder},
        crypto::generate_keypair_dummy,
        miner::Miner,
        script::{OpCode, Script, ScriptItem},
        serialization::BitcoinSerialize,
        tests::dummy_tx::get_valid_tx,
    };
    use tokio::{
        sync::RwLock,
        task::JoinHandle,
        time::{Duration, Instant, sleep},
    };

    use crate::{
        network::{
            Command, NetworkMessage, OutboundManager, PingMessage, config::GENSIS_CONFIG,
            server::NetworkServer,
        },
        node::Node,
    };

    use std::{
        collections::HashSet,
        env, fs,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    /// Helper function to create a temporary database path
    fn create_temp_db_path(test_name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "btc-node-{}-{}-{}.redb",
            test_name,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    async fn spawn_bootstrap_servers() -> Vec<JoinHandle<()>> {
        let mut server_tasks = Vec::new();

        for port in 3001..=3005 {
            let node = Arc::new(RwLock::new(
                Node::new(
                    create_temp_db_path(&format!("bootstrap-{port}")),
                    Some(GENSIS_CONFIG),
                    &format!("127.0.0.1:{port}"),
                )
                .await
                .expect("bootstrap node should initialize"),
            ));
            let server = node.read().await.server.clone();

            server_tasks.push(tokio::spawn(async move {
                server.run(node).await.expect("bootstrap server should run");
            }));
        }

        server_tasks
    }

    async fn spawn_node_server(node: Arc<RwLock<Node>>) -> JoinHandle<()> {
        let server = node.read().await.server.clone();
        tokio::spawn(async move {
            server.run(node).await.expect("node server should run");
        })
    }

    fn miner_script() -> Script {
        let (_, public_key) = generate_keypair_dummy();
        Script {
            items: vec![
                ScriptItem::Op(OpCode::Dup),
                ScriptItem::Op(OpCode::Hash160),
                ScriptItem::PushData(
                    btc_core::crypto::hash::hash160(&public_key.serialize().to_vec()).to_vec(),
                ),
                ScriptItem::Op(OpCode::EqualVerify),
                ScriptItem::Op(OpCode::CheckSig),
            ],
        }
    }

    fn mine_blocks(node: &mut Node, count: usize, script: &Script) -> Vec<Block> {
        let mut blocks = Vec::with_capacity(count);
        for _ in 0..count {
            let mut block = Builder::build(&[], script.clone(), &node.chain).unwrap();
            let parent = node.chain.tip_node().unwrap();
            block.header.timestamp = node.chain.median_timestamp(parent).unwrap() + 1;
            Miner::mine(&mut block).unwrap();
            node.chain.add_block(block.clone()).unwrap();
            blocks.push(block);
        }
        blocks
    }

    async fn wait_for_height(node: &Arc<RwLock<Node>>, expected_height: u32) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if node.read().await.chain.height() >= expected_height {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "node did not reach height {expected_height}"
            );
            sleep(Duration::from_millis(50)).await;
        }
    }

    /// Test: Node initialization with genesis configuration
    /// Verifies that a node can be created and properly initialized with the shared genesis config
    #[tokio::test]
    async fn test_node_initialization_with_genesis_config() {
        let path = create_temp_db_path("node-init");
        let _ = fs::remove_file(&path);

        let port = "127.0.0.1:0";

        // Create a node with the shared genesis configuration
        let node = Node::new(&path, Some(GENSIS_CONFIG), port)
            .await
            .expect("node should initialize");

        // Verify the node is at genesis height
        let tip = node.chain.tip_node().expect("genesis tip should exist");
        assert_eq!(node.chain.height(), 0, "chain should start at height 0");
        assert!(tip.parent.is_none(), "genesis block should have no parent");

        let _ = fs::remove_file(&path);
    }

    /// Test: Basic network server startup
    /// Verifies that a network server can bind to an address
    #[tokio::test]
    async fn test_network_server_binding() {
        let _server: NetworkServer = NetworkServer::bind("127.0.0.1:0")
            .await
            .expect("server should bind to any available port");

        // Server is successfully bound and listening
    }

    #[tokio::test]
    async fn p2p_communication() {
        let port = "127.0.0.1:0";
        let path = create_temp_db_path("test_3");
        let path2 = create_temp_db_path("test_2");

        let node_a = Node::new(path.clone(), Some(GENSIS_CONFIG), port)
            .await
            .expect("node creation error");

        let node_b = Node::new(path2, Some(GENSIS_CONFIG), port)
            .await
            .expect("node creation error");
        let bootstrap_servers = spawn_bootstrap_servers().await;

        let mut outbound_manager = OutboundManager::new();

        let node_ob_mgr = Arc::new(RwLock::new(node_a));
        let node_b_ob_mgr = Arc::new(RwLock::new(node_b));
        let node_b_server = spawn_node_server(node_b_ob_mgr.clone()).await;

        outbound_manager
            .connect_bootstrap(node_ob_mgr.clone())
            .await
            .unwrap();

        let peer_count = node_ob_mgr.read().await.manager.read().await.peers.len();

        let soc_addr_b = node_b_ob_mgr
            .read()
            .await
            .server
            .listener
            .local_addr()
            .unwrap();
        assert_eq!(peer_count, 5);

        let _peer = outbound_manager
            .connect_to_peer(soc_addr_b, node_ob_mgr.clone())
            .await
            .unwrap();
        let peer_count = node_ob_mgr.read().await.manager.read().await.peers.len();
        assert_eq!(peer_count, 6);

        let tx = {
            let mut node = node_ob_mgr.write().await;
            get_valid_tx(&mut node.chain.ledger, 50, 0, 40)
        };
        let utxo = node_ob_mgr
            .read()
            .await
            .chain
            .ledger
            .get_utxo(&tx.inputs[0].previous_output)
            .unwrap()
            .clone();
        node_b_ob_mgr
            .write()
            .await
            .chain
            .ledger
            .add_utxo(tx.inputs[0].previous_output.clone(), utxo)
            .unwrap();

        node_ob_mgr
            .write()
            .await
            .submit_transaction(tx.clone(), HashSet::new())
            .await
            .unwrap();

        let mut is_tx_in_node_b = false;
        for _ in 0..20 {
            is_tx_in_node_b = node_b_ob_mgr
                .read()
                .await
                .chain
                .mempool
                .get_transaction(&tx.txid())
                .is_some();
            if is_tx_in_node_b {
                break;
            }
            sleep(Duration::from_millis(50)).await;
        }

        assert!(is_tx_in_node_b);

        node_b_server.abort();
        let _ = node_b_server.await;

        for server_task in bootstrap_servers {
            server_task.abort();
        }
    }

    #[tokio::test]
    async fn integration_sync_and_relay_between_nodes() {
        let script = miner_script();
        let node_a = Arc::new(RwLock::new(
            Node::new(
                create_temp_db_path("integration-a"),
                Some(GENSIS_CONFIG),
                "127.0.0.1:0",
            )
            .await
            .unwrap(),
        ));

        let first_five = {
            let mut node = node_a.write().await;
            mine_blocks(&mut node, 5, &script)
        };

        let node_b = Arc::new(RwLock::new(
            Node::new(
                create_temp_db_path("integration-b"),
                Some(GENSIS_CONFIG),
                "127.0.0.1:0",
            )
            .await
            .unwrap(),
        ));
        {
            let mut node = node_b.write().await;
            for block in &first_five {
                node.chain.add_block(block.clone()).unwrap();
            }
            assert_eq!(node.chain.height(), 5);
        }

        {
            let mut node = node_a.write().await;
            mine_blocks(&mut node, 5, &script);
            assert_eq!(node.chain.height(), 10);
        }

        let node_b_tip = node_b.read().await.chain.tip_node().unwrap().hash;
        assert_eq!(first_five[4].header.hash(), node_b_tip);
        assert_eq!(
            node_a
                .read()
                .await
                .headers_for_locators(vec![node_b_tip])
                .len(),
            5
        );

        let node_a_server = spawn_node_server(node_a.clone()).await;
        let peer_address = node_a.read().await.server.listener.local_addr().unwrap();
        let mut outbound_manager = OutboundManager::new();
        outbound_manager
            .connect_to_peer(peer_address, node_b.clone())
            .await
            .unwrap();

        wait_for_height(&node_b, 10).await;
        assert_eq!(
            node_a.read().await.chain.tip_node().unwrap().hash,
            node_b.read().await.chain.tip_node().unwrap().hash
        );

        let tx = {
            let mut node = node_a.write().await;
            let tx = get_valid_tx(&mut node.chain.ledger, 50, 0, 40);
            let utxo = node
                .chain
                .ledger
                .get_utxo(&tx.inputs[0].previous_output)
                .unwrap()
                .clone();
            node_b
                .write()
                .await
                .chain
                .ledger
                .add_utxo(tx.inputs[0].previous_output.clone(), utxo)
                .unwrap();
            tx
        };
        node_a
            .write()
            .await
            .submit_transaction(tx.clone(), HashSet::new())
            .await
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while node_b
            .read()
            .await
            .chain
            .mempool
            .get_transaction(&tx.txid())
            .is_none()
        {
            assert!(Instant::now() < deadline, "transaction did not propagate");
            sleep(Duration::from_millis(50)).await;
        }

        let block = {
            let mut node = node_a.write().await;
            mine_blocks(&mut node, 1, &script).pop().unwrap()
        };
        node_a
            .write()
            .await
            .submit_block(block.clone(), HashSet::new())
            .await
            .unwrap();
        wait_for_height(&node_b, 11).await;
        assert_eq!(
            node_b.read().await.chain.tip_node().unwrap().hash,
            block.header.hash()
        );

        let (ping_sender, ping_state) = {
            let node = node_a.read().await;
            let manager = node.manager.read().await;
            let peer = manager.peers.values().next().unwrap();
            (peer.sender.clone(), peer.ping_state.clone())
        };
        let nonce = 42;
        {
            let mut state = ping_state.lock().await;
            state.nonce = Some(nonce);
            state.sent_at = Some(Instant::now());
        }
        ping_sender
            .send(NetworkMessage {
                command: Command::Ping,
                payload: PingMessage { nonce }.serialize(),
            })
            .await
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while ping_state.lock().await.nonce.is_some() {
            assert!(Instant::now() < deadline, "pong was not received");
            sleep(Duration::from_millis(50)).await;
        }

        node_a_server.abort();
        let _ = node_a_server.await;
    }
}
