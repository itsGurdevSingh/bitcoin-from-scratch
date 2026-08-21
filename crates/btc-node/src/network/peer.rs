use std::net::SocketAddr;

use btc_core::{
    serialization::{BitcoinDeserialize, BitcoinSerialize},
    utils::time::Time,
};
use rand::random;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::network::{
    Command, NetworkMessage, VersionMessage,
    config::{
        ALLOWED_SERVICES, MIN_PEER_PROTOCOL_VERSION, NODE_NETWORK, PROTOCOL_VERSION, USER_AGENT,
    },
    error::PeerError,
    message::NetworkMessageHeader,
    version_message::NetworkAddress,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionDirection {
    Inbound,
    Outbound,
}

pub enum PeerState {
    Handshaking,
    Active,
}

pub type PeerId = u64;
pub struct Peer {
    pub id: PeerId,
    pub address: SocketAddr,
    pub direction: ConnectionDirection,
    pub state: PeerState,
    pub version: Option<VersionMessage>,
    pub stream: TcpStream,
}

impl Peer {
    pub fn new(stream: TcpStream, address: SocketAddr, direction: ConnectionDirection) -> Self {
        Self {
            stream,
            address,
            direction,
            id: random::<u64>(),
            state: PeerState::Handshaking,
            version: None,
        }
    }

    pub async fn handshake(&mut self) -> Result<(), PeerError> {
        self.send_version_message().await?;

        let version = self.read_version_message().await?;

        self.verify_version_message(&version)?;

        self.version = Some(version);

        self.send_verack().await?;

        self.read_verack().await?;

        self.state = PeerState::Active;

        Ok(())
    }

    pub fn verify_version_message(
        &mut self,
        version_message: &VersionMessage,
    ) -> Result<(), PeerError> {
        if version_message.version < MIN_PEER_PROTOCOL_VERSION {
            Err(PeerError::UnexpectedVersion)?
        }

        if !ALLOWED_SERVICES.contains(&version_message.services) {
            Err(PeerError::UnexpectedService)?
        }

        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<NetworkMessage, PeerError> {
        let mut header_bytes = [0u8; 24];

        self.stream
            .read_exact(&mut header_bytes)
            .await
            .map_err(|_| PeerError::Io)?;

        let (header, _) =
            NetworkMessageHeader::deserialize(&header_bytes).map_err(PeerError::Deserialize)?;

        let mut payload = vec![0u8; header.payload_len];

        self.stream
            .read_exact(&mut payload)
            .await
            .map_err(|_| PeerError::Io)?;

        Ok(NetworkMessage {
            command: header.command,
            payload: payload.to_vec(),
        })
    }

    pub async fn read_version_message(&mut self) -> Result<VersionMessage, PeerError> {
        let message = self.read_message().await?;
        // command need to be version
        if message.command != Command::Version {
            return Err(PeerError::UnexpectedCommand);
        }
        let (version, _) =
            VersionMessage::deserialize(&message.payload).map_err(PeerError::Deserialize)?;
        Ok(version)
    }

    pub async fn send_version_message(&mut self) -> Result<(), PeerError> {
        let version = VersionMessage {
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
            nonce: random::<u64>(),
            user_agent: USER_AGENT.to_string(),
            start_height: 0,
            relay: true,
        };

        let message = NetworkMessage {
            command: Command::Version,
            payload: version.serialize(),
        };
        self.stream.write_all(&message.serialize()).await.unwrap();
        Ok(())
    }

    pub async fn send_verack(&mut self) -> Result<(), PeerError> {
        self.stream
            .write_all(
                &NetworkMessage {
                    command: Command::Verack,
                    payload: Vec::new(),
                }
                .serialize(),
            )
            .await
            .unwrap();
        Ok(())
    }

    pub async fn read_verack(&mut self) -> Result<(), PeerError> {
        let message = self.read_message().await?;
        // command need to be verack
        if message.command != Command::Verack {
            return Err(PeerError::UnexpectedCommand);
        }
        Ok(())
    }
}
