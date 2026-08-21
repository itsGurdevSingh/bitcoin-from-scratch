use std::{collections::HashMap, net::SocketAddr, sync::Arc,};

use tokio::{net::TcpStream, sync::RwLock};

use crate::network::{Peer, peer::{ConnectionDirection, PeerId, PeerState}};

pub struct PeerManager {
    pub peers: HashMap<PeerId, Peer>,
}

impl PeerManager {
    pub fn new() -> Self {
        Self { peers: HashMap::new()}
    }

   pub async fn process_connection(
    manager: Arc<RwLock<Self>>,
    stream: TcpStream,
    address: SocketAddr,
) {
    let mut peer = Peer::new(
        stream,
        address,
        ConnectionDirection::Inbound,
    );

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