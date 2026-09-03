use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use tokio::{net::TcpStream, sync::RwLock};

use crate::network::error::PeerError;
use crate::network::{Peer, PeerManager, peer::ConnectionDirection};
use crate::node::Node;

const BOOTSTRAP_PEERS: [&str; 5] = [
    "127.0.0.1:3001",
    "127.0.0.1:3002",
    "127.0.0.1:3003",
    "127.0.0.1:3004",
    "127.0.0.1:3005",
];

pub struct OutboundManager {
    connected: HashSet<String>,
}

impl OutboundManager {
    pub fn new() -> Self {
        Self {
            connected: HashSet::new(),
        }
    }

    pub async fn connect_bootstrap(&mut self, node: Arc<RwLock<Node>>) -> Result<(), PeerError> {
        self.connect_to_peers(BOOTSTRAP_PEERS, node).await
    }

    pub async fn connect_to_peer(
        &mut self,
        peer_addr: SocketAddr,
        node: Arc<RwLock<Node>>,
    ) -> Result<(), PeerError> {
        let address_key = peer_addr.to_string();
        if self.connected.contains(&address_key) {
            return Ok(());
        }

        let stream = TcpStream::connect(peer_addr)
            .await
            .map_err(|_| PeerError::Io)?;
        let peer = Peer::new(stream, peer_addr, ConnectionDirection::Outbound);

        PeerManager::process_peer(peer, node).await;
        self.connected.insert(address_key);
        Ok(())
    }

    async fn connect_to_peers<'a, I>(
        &mut self,
        peers: I,
        node: Arc<RwLock<Node>>,
    ) -> Result<(), PeerError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        for peer_addr in peers {
            let address = match peer_addr.parse::<SocketAddr>() {
                Ok(address) => address,
                Err(_) => continue,
            };

            match self.connect_to_peer(address, Arc::clone(&node)).await {
                Ok(()) => {}
                Err(_err) => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::network::Command;
    use btc_core::serialization::{BitcoinDeserialize, BitcoinSerialize};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        sync::RwLock,
    };

    use crate::{
        network::{NetworkMessage, message::NetworkMessageHeader, peer::ConnectionDirection},
        node::Node,
    };

    use super::*;

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

    async fn spawn_valid_handshake_server() -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            let inbound_version = read_network_message(&mut stream).await;
            assert_eq!(inbound_version.command, Command::Version);

            write_network_message(
                &mut stream,
                NetworkMessage {
                    command: Command::Version,
                    payload: inbound_version.payload,
                },
            )
            .await;

            let inbound_verack = read_network_message(&mut stream).await;
            assert_eq!(inbound_verack.command, Command::Verack);

            write_network_message(
                &mut stream,
                NetworkMessage {
                    command: Command::Verack,
                    payload: Vec::new(),
                },
            )
            .await;
        });

        (address.to_string(), handle)
    }

    #[tokio::test]
    async fn connect_to_peers_succeeds_handshakes() {
        let (peer_a, task_a) = spawn_valid_handshake_server().await;
        let (peer_b, task_b) = spawn_valid_handshake_server().await;
        let path = env::temp_dir().join(format!(
            "btc-node-outbound-{}-{}.redb",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let node = Arc::new(RwLock::new(
            Node::new(&path, None, "127.0.0.1:0").await.unwrap(),
        ));

        let mut manager = OutboundManager::new();
        let result = manager
            .connect_to_peers(vec![peer_a.as_str(), peer_b.as_str()], Arc::clone(&node))
            .await;

        assert!(result.is_ok());

        task_a.await.unwrap();
        task_b.await.unwrap();

        assert_eq!(manager.connected.len(), 2);
        assert!(manager.connected.contains(&peer_a));
        assert!(manager.connected.contains(&peer_b));

        let node_binding = node.read().await;
        let peer_manager = node_binding.manager.read().await;
        assert_eq!(peer_manager.peers.len(), 2);
        assert!(
            peer_manager
                .peers
                .values()
                .all(|peer| peer.direction == ConnectionDirection::Outbound)
        );

        let _ = std::fs::remove_file(path);
    }
}
