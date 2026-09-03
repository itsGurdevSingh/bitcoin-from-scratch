use std::sync::Arc;

use btc_core::{
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    types::BlockHash,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::{
        Mutex,
        mpsc::{self, Sender},
    },
};

use crate::network::{
    Command, GetDataMessage, GetHeadersMessage, InventoryType, InventoryVector, NetworkHandler,
    NetworkMessage, Peer, PeerHandle, PingMessage, PongMessage, config::HEADERS_MAX_BATCH_SIZE,
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
            network_handler.clone(),
        ));

        tokio::spawn(Self::writer_loop(writer, receiver));

        let _res = Self::sync_headers(sender.clone(), network_handler).await;

        PeerHandle {
            sender,
            ping_state: state,
            direction: peer.direction,
            relay: peer.version.is_some_and(|version| version.relay),
        }
    }

    pub async fn sync_headers(
        sender: Sender<NetworkMessage>,
        network_handler: Arc<NetworkHandler>,
    ) -> Result<(), PeerError> {
        let locator_hashes = network_handler
            .build_locator()
            .await
            .map_err(PeerError::Network)?;
        let get_headers = GetHeadersMessage::new(locator_hashes, BlockHash([0u8; 32]));
        let network_message = NetworkMessage {
            command: Command::GetHeaders,
            payload: get_headers.serialize(),
        };
        let _res = sender.send(network_message).await;
        Ok(())
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
            let message = match Self::read_message(&mut reader).await {
                Ok(message) => message,
                Err(_) => break,
            };
            let _ = Self::handle_message(
                message,
                sender.clone(),
                state.clone(),
                network_handler.clone(),
            )
            .await;
        }
    }

    async fn writer_loop(
        mut writer: OwnedWriteHalf,
        mut receiver: mpsc::Receiver<NetworkMessage>,
    ) {
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

            Command::GetHeaders => {
                let headers_message = network_handler
                    .handle_get_headers(message.payload)
                    .await
                    .map_err(PeerError::Network)?;

                let network_message = NetworkMessage {
                    command: Command::Headers,
                    payload: headers_message.serialize(),
                };

                let _res = sender.send(network_message).await;
                Ok(())
            }

            Command::Headers => {
                let block_hashes = network_handler
                    .handle_headers(message.payload)
                    .await
                    .map_err(PeerError::Network)?;

                let mut inventory: Vec<InventoryVector> = Vec::new();
                for block_hash in block_hashes.iter() {
                    let inv_vec = InventoryVector {
                        inv_type: InventoryType::Block,
                        hash: block_hash.into_bytes(),
                    };
                    inventory.push(inv_vec);
                }

                let inv_message = GetDataMessage { inventory };
                if !inv_message.inventory.is_empty() {
                    for inv_vec in &inv_message.inventory {
                        network_handler.mark_requesing_data(inv_vec.clone()).await;
                    }
                    let network_message = NetworkMessage {
                        command: Command::GetData,
                        payload: inv_message.serialize(),
                    };
                    let _res = sender.send(network_message).await;
                }

                 if block_hashes.len() >= HEADERS_MAX_BATCH_SIZE {
                    // our sync headers called agian for confirmation of furtehr headers recival.
                    Self::sync_headers(sender.clone(), network_handler.clone()).await?;
                }

                Ok(())
                // later
            }

            Command::Inv => {
                let get_data_message = network_handler.check_inventory(message.payload).await;

                if get_data_message.inventory.is_empty() {
                    return Ok(());
                };

                for inv_vec in &get_data_message.inventory {
                    network_handler.mark_requesing_data(inv_vec.clone()).await;
                }

                let _ = sender
                    .send(NetworkMessage {
                        command: Command::GetData,
                        payload: get_data_message.serialize(),
                    })
                    .await;
                Ok(())
            }

            Command::GetData => {
                let (get_data_msg, _) = GetDataMessage::deserialize(&message.payload)
                    .map_err(PeerError::Deserialize)?;

                for inv_vec in get_data_msg.inventory {
                    match inv_vec.inv_type {
                        InventoryType::Block => {
                            let block = network_handler
                                .get_block(inv_vec.hash)
                                .await
                                .map_err(PeerError::Network)?;
                            let _ = sender
                                .send(NetworkMessage {
                                    command: Command::Block,
                                    payload: block.serialize(),
                                })
                                .await;
                        }
                        InventoryType::Tx => {
                            let tx = network_handler
                                .get_tx(inv_vec.hash)
                                .await
                                .map_err(PeerError::Network)?;
                            let _ = sender
                                .send(NetworkMessage {
                                    command: Command::Tx,
                                    payload: tx.serialize(),
                                })
                                .await;
                        }
                    }
                }
                Ok(())
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
