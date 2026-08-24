use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::{
    net::TcpStream,
    sync::{RwLock, mpsc::Sender},
};

use crate::{
    network::{
        Command, NetworkHandler, NetworkMessage, Peer, PeerConnection, PeerHandle,
        peer::{ConnectionDirection, PeerId, PeerState},
    },
    node::Node,
};

pub struct PeerManager {
    pub peers: HashMap<PeerId, PeerHandle>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub async fn process_connection(
        stream: TcpStream,
        address: SocketAddr,
        node: Arc<RwLock<Node>>,
    ) {
        let mut peer = Peer::new(stream, address, ConnectionDirection::Inbound);

        // No manager lock here.
        if peer.handshake().await.is_err() {
            return;
        }

        peer.state = PeerState::Active;
        let id = peer.id;

        let manager = Arc::clone(&node.read().await.manager);
        let network_handler = NetworkHandler::new(node, peer.id);
        let handle = PeerConnection::start_peer(peer, Arc::new(network_handler)).await;

        // Only lock when modifying the peer map.
        let mut m = manager.write().await;

        m.add_handle(id, handle);
    }

    fn add_handle(&mut self, peer_id: PeerId, handle: PeerHandle) {
        self.peers.insert(peer_id, handle);
    }

    pub async fn broadcast_transaction(&self, tx: Vec<u8>, origin_peer: Option<PeerId>) {
        let message: NetworkMessage = NetworkMessage {
            command: Command::Tx,
            payload: tx,
        };

        for (id, handle) in self.peers.iter() {
            if handle.relay == false || Some(*id) == origin_peer {
                continue;
            };
            let _ = handle.sender.send(message.clone()).await;
        }
    }

    pub async fn broadcast_block(&self, block: Vec<u8>, origin_peer: Option<PeerId>) {
        let message: NetworkMessage = NetworkMessage {
            command: Command::Block,
            payload: block,
        };

        for (id, handle) in self.peers.iter() {
            if Some(*id) == origin_peer {
                continue;
            };
            let _ = handle.sender.send(message.clone()).await;
        }
    }

    pub fn broadcast_transaction_handles(
        &self,
        origin_peer: Option<PeerId>,
    ) -> Vec<Sender<NetworkMessage>> {
        self.peers
            .iter()
            .filter(|(id, handle)| handle.relay && Some(**id) != origin_peer)
            .map(|(_, handle)| handle.sender.clone())
            .collect()
    }

    pub fn broadcast_block_handles(
        &self,
        origin_peer: Option<PeerId>,
    ) -> Vec<Sender<NetworkMessage>> {
        self.peers
            .iter()
            .filter(|(id, _)| Some(**id) != origin_peer)
            .map(|(_, handle)| handle.sender.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use btc_core::{
        serialization::{BitcoinDeserialize, BitcoinSerialize},
        utils::time::Time,
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
        sync::RwLock,
        time::{Duration, sleep},
    };

    use crate::{
        network::{
            Command, NetworkMessage,
            config::{NODE_NETWORK, PROTOCOL_VERSION, USER_AGENT},
            message::NetworkMessageHeader,
            version_message::{NetworkAddress, VersionMessage},
        },
        node::Node,
    };

    use super::PeerManager;

    async fn read_network_message(stream: &mut TcpStream) -> NetworkMessage {
        let mut header_bytes = [0u8; 24];
        stream.read_exact(&mut header_bytes).await.unwrap();

        let (header, _) = NetworkMessageHeader::deserialize(&header_bytes).unwrap();

        let mut payload = vec![0u8; header.payload_len];
        stream.read_exact(&mut payload).await.unwrap();

        NetworkMessage {
            command: header.command,
            payload: payload.to_vec(),
        }
    }

    async fn write_network_message(stream: &mut TcpStream, message: NetworkMessage) {
        stream.write_all(&message.serialize()).await.unwrap();
    }

    fn build_version_message() -> VersionMessage {
        VersionMessage {
            version: PROTOCOL_VERSION,
            services: NODE_NETWORK,
            timestamp: Time::unix_timestamp(),
            receiver: NetworkAddress {
                services: NODE_NETWORK,
                ip: [0u8; 16],
                port: 3000,
            },
            sender: NetworkAddress {
                services: NODE_NETWORK,
                ip: [0u8; 16],
                port: 3000,
            },
            nonce: 42,
            user_agent: USER_AGENT.to_string(),
            start_height: 0,
            relay: true,
        }
    }

    fn test_db_path(name: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        env::temp_dir().join(format!("btc-node-{name}-{unique_suffix}.redb"))
    }

    #[tokio::test]
    async fn inbound_connection_handshake_success_adds_active_peer() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let path = test_db_path("test_db");
        let node = Node::new(path).unwrap();
        let node_for_server = Arc::new(RwLock::new(node));
        let node_for_server_clone = node_for_server.clone();

        let server_task = tokio::spawn(async move {
            let (stream, peer_address) = listener.accept().await.unwrap();
            PeerManager::process_connection(stream, peer_address, node_for_server).await;
        });

        let mut client = TcpStream::connect(address).await.unwrap();

        let server_version = read_network_message(&mut client).await;
        assert_eq!(server_version.command, Command::Version);

        write_network_message(
            &mut client,
            NetworkMessage {
                command: Command::Version,
                payload: build_version_message().serialize(),
            },
        )
        .await;

        let server_verack = read_network_message(&mut client).await;
        assert_eq!(server_verack.command, Command::Verack);

        write_network_message(
            &mut client,
            NetworkMessage {
                command: Command::Verack,
                payload: Vec::new(),
            },
        )
        .await;

        server_task.await.unwrap();

        let node_binding = node_for_server_clone.read().await;
        let manager = node_binding.manager.read().await;
        assert_eq!(manager.peers.len(), 1);

        let peer = manager.peers.values().next().unwrap();
        assert_eq!(
            peer.direction,
            crate::network::peer::ConnectionDirection::Inbound
        );
    }

    #[tokio::test]
    async fn inbound_connection_handshake_failure_does_not_add_peer() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let path = test_db_path("test_db");
        let node = Node::new(path).unwrap();
        let node_for_server = Arc::new(RwLock::new(node));
        let node_for_server_clone = node_for_server.clone();

        let server_task = tokio::spawn(async move {
            let (stream, peer_address) = listener.accept().await.unwrap();
            PeerManager::process_connection(stream, peer_address, node_for_server).await;
        });

        let mut client = TcpStream::connect(address).await.unwrap();

        let server_version = read_network_message(&mut client).await;
        assert_eq!(server_version.command, Command::Version);

        // Send unexpected command where a Version message is required.
        write_network_message(
            &mut client,
            NetworkMessage {
                command: Command::Ping,
                payload: Vec::new(),
            },
        )
        .await;

        server_task.await.unwrap();

        let node_binding = node_for_server_clone.read().await;
        let manager = node_binding.manager.read().await;
        assert_eq!(manager.peers.len(), 0);
    }

    #[tokio::test]
    async fn inbound_connection_handshake_succeeds_with_delayed_client_verack() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let path = test_db_path("test_db2");
        let node = Node::new(path).unwrap();
        let node_for_server = Arc::new(RwLock::new(node));
        let node_for_server_clone = node_for_server.clone();

        let server_task = tokio::spawn(async move {
            let (stream, peer_address) = listener.accept().await.unwrap();
            PeerManager::process_connection(stream, peer_address, node_for_server).await;
        });

        let mut client = TcpStream::connect(address).await.unwrap();

        let server_version = read_network_message(&mut client).await;
        assert_eq!(server_version.command, Command::Version);

        write_network_message(
            &mut client,
            NetworkMessage {
                command: Command::Version,
                payload: build_version_message().serialize(),
            },
        )
        .await;

        let server_verack = read_network_message(&mut client).await;
        assert_eq!(server_verack.command, Command::Verack);

        // Simulate a small network delay before the peer completes the handshake.
        sleep(Duration::from_secs(1)).await;

        write_network_message(
            &mut client,
            NetworkMessage {
                command: Command::Verack,
                payload: Vec::new(),
            },
        )
        .await;

        server_task.await.unwrap();

        let node_binding = node_for_server_clone.read().await;
        let manager = node_binding.manager.read().await;
        assert_eq!(manager.peers.len(), 1);
    }
}
