use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};

use crate::network::{NetworkMessage, peer::{ConnectionDirection, PingState}};

pub struct PeerHandle {
    pub sender: mpsc::Sender<NetworkMessage>,
    pub ping_state: Arc<Mutex<PingState>>,
    pub direction: ConnectionDirection,
    pub relay: bool
}
