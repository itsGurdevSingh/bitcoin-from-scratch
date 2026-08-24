use btc_core::serialization::DeserializeError;

use crate::node::NodeError;

#[derive(Debug)]
pub enum NetworkError {
    TypeCastFailed,
    Node(NodeError),
    Deserialize(DeserializeError)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDeserializeError {
    UnexpectedEndOfBytes { expected: usize, actual: usize },
    InvalidMagic,
    InvalidCommand,
    InvalidPayloadLength { expected: usize, actual: usize },
    InvalidChecksum,
    PayloadTooLarge,
    InvalidType
}

#[derive(Debug)]
pub enum PeerError {
    UnexpectedCommand,
    UnexpectedVersion,
    UnexpectedService,
    UnsupportedUserAgent,
    Io,
    Deserialize(NetworkDeserializeError),
    Network(NetworkError)
}
