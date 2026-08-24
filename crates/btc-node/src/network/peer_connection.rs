use std::sync::Arc;

use btc_core::serialization::{BitcoinDeserialize, BitcoinSerialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::{
        Mutex,
        mpsc::{self, Sender},
    },
};

use crate::network::{
    Command, NetworkHandler, NetworkMessage, Peer, PeerHandle, PingMessage, PongMessage,
    error::PeerError, message::NetworkMessageHeader, peer::PingState,
};

pub struct PeerConnection;

impl PeerConnection {
    pub async fn start_peer(peer: Peer, network_handler: Arc<NetworkHandler>) -> PeerHandle {
        let (reader, writer) = peer.stream.into_split();

        let (sender, receiver) = mpsc::channel(100);

        let state = Arc::new(Mutex::new(PingState::new()));

        tokio::spawn(Self::reader_loop(
            reader,
            sender.clone(),
            state.clone(),
            network_handler,
        ));

        tokio::spawn(Self::writer_loop(writer, receiver));

        PeerHandle {
            sender,
            ping_state: state,
            direction: peer.direction,
            relay: peer.version.is_some_and(|version| version.relay),
        }
    }

    pub async fn read_message(reader: &mut OwnedReadHalf) -> Result<NetworkMessage, PeerError> {
        let mut header_bytes = [0u8; 24];

        reader
            .read_exact(&mut header_bytes)
            .await
            .map_err(|_| PeerError::Io)?;

        let (header, _) =
            NetworkMessageHeader::deserialize(&header_bytes).map_err(PeerError::Deserialize)?;

        let mut payload = vec![0u8; header.payload_len];

        reader
            .read_exact(&mut payload)
            .await
            .map_err(|_| PeerError::Io)?;

        Ok(NetworkMessage {
            command: header.command,
            payload: payload.to_vec(),
        })
    }

    pub async fn reader_loop(
        mut reader: OwnedReadHalf,
        sender: Sender<NetworkMessage>,
        state: Arc<Mutex<PingState>>,
        network_handler: Arc<NetworkHandler>,
    ) {
        loop {
            let message = Self::read_message(&mut reader).await.unwrap();
            let _ = Self::handle_message(
                message,
                sender.clone(),
                state.clone(),
                network_handler.clone(),
            )
            .await;
        }
    }

    async fn writer_loop(mut writer: OwnedWriteHalf, mut receiver: mpsc::Receiver<NetworkMessage>) {
        while let Some(message) = receiver.recv().await {
            let _ = writer.write_all(&message.serialize()).await;
        }
    }

    pub async fn handle_message(
        message: NetworkMessage,
        sender: Sender<NetworkMessage>,
        state: Arc<Mutex<PingState>>,
        network_handler: Arc<NetworkHandler>,
    ) -> Result<(), PeerError> {
        match message.command {
            Command::Ping => Self::handle_ping(message, sender).await,

            Command::Pong => Self::handle_pong(message, state).await,

            Command::Tx => {
                *&network_handler
                    .handle_transaction(message.payload)
                    .await
                    .map_err(PeerError::Network)?;
                Ok(())
            }

            Command::Block => {
                // later
                *&network_handler
                    .handle_block(message.payload)
                    .await
                    .map_err(PeerError::Network)?;
                Ok(())
            }

            Command::Headers => {
                Ok(())
                // later
            }

            _ => Ok(()),
        }
    }

    pub async fn handle_ping(
        message: NetworkMessage,
        sender: Sender<NetworkMessage>,
    ) -> Result<(), PeerError> {
        let (ping, _) =
            PingMessage::deserialize(&message.payload).map_err(PeerError::Deserialize)?;
        let pong = PongMessage { nonce: ping.nonce };
        let message: NetworkMessage = NetworkMessage {
            command: Command::Pong,
            payload: pong.serialize(),
        };
        // send that messege to our sender.
        let _ = sender.send(message).await;
        Ok(())
    }

    pub async fn handle_pong(
        message: NetworkMessage,
        state: Arc<Mutex<PingState>>,
    ) -> Result<(), PeerError> {
        let (pong, _) =
            PongMessage::deserialize(&message.payload).map_err(PeerError::Deserialize)?;

        let mut guard = state.lock().await;

        match (guard.nonce, guard.sent_at) {
            (Some(nonce), Some(sent_at)) if nonce == pong.nonce => {
                // clear after successful pong
                guard.nonce = None;
                guard.sent_at = None;
            }
            _ => {}
        };
        Ok(())
    }
}
