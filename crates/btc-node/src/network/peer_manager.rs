use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::{net::TcpStream, sync::RwLock};

use crate::network::{
    Peer,
    peer::{ConnectionDirection, PeerId, PeerState},
};

pub struct PeerManager {
    pub peers: HashMap<PeerId, Peer>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub async fn process_connection(
        manager: Arc<RwLock<Self>>,
        stream: TcpStream,
        address: SocketAddr,
    ) {
        let mut peer = Peer::new(stream, address, ConnectionDirection::Inbound);

        // No manager lock here.
        if peer.handshake().await.is_err() {
            return;
        }

        peer.state = PeerState::Active;

        // Only lock when modifying the peer map.
        let mut manager = manager.write().await;

        manager.add_peer(peer);
    }

    fn add_peer(&mut self, peer: Peer) {
        self.peers.insert(peer.id, peer);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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

    use crate::network::{
        Command, NetworkMessage,
        config::{NODE_NETWORK, PROTOCOL_VERSION, USER_AGENT},
        message::NetworkMessageHeader,
        peer::PeerState,
        version_message::{NetworkAddress, VersionMessage},
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

    #[tokio::test]
    async fn inbound_connection_handshake_success_adds_active_peer() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let manager = Arc::new(RwLock::new(PeerManager::new()));
        let manager_for_server = Arc::clone(&manager);

        let server_task = tokio::spawn(async move {
            let (stream, peer_address) = listener.accept().await.unwrap();
            PeerManager::process_connection(manager_for_server, stream, peer_address).await;
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

        let manager = manager.read().await;
        assert_eq!(manager.peers.len(), 1);

        let peer = manager.peers.values().next().unwrap();
        assert_eq!(
            peer.direction,
            crate::network::peer::ConnectionDirection::Inbound
        );
        assert!(matches!(peer.state, PeerState::Active));
    }

    #[tokio::test]
    async fn inbound_connection_handshake_failure_does_not_add_peer() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let manager = Arc::new(RwLock::new(PeerManager::new()));
        let manager_for_server = Arc::clone(&manager);

        let server_task = tokio::spawn(async move {
            let (stream, peer_address) = listener.accept().await.unwrap();
            PeerManager::process_connection(manager_for_server, stream, peer_address).await;
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

        let manager = manager.read().await;
        assert_eq!(manager.peers.len(), 0);
    }

    #[tokio::test]
    async fn inbound_connection_handshake_succeeds_with_delayed_client_verack() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let manager = Arc::new(RwLock::new(PeerManager::new()));
        let manager_for_server = Arc::clone(&manager);

        let server_task = tokio::spawn(async move {
            let (stream, peer_address) = listener.accept().await.unwrap();
            PeerManager::process_connection(manager_for_server, stream, peer_address).await;
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

        let manager = manager.read().await;
        assert_eq!(manager.peers.len(), 1);
        let peer = manager.peers.values().next().unwrap();
        assert!(matches!(peer.state, PeerState::Active));
    }
}
