#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    TypeCastFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDeserializeError {
    UnexpectedEndOfBytes { expected: usize, actual: usize },
    InvalidMagic,
    InvalidCommand,
    InvalidPayloadLength { expected: usize, actual: usize },
    InvalidChecksum,
    PayloadTooLarge
}

pub enum PeerError {
    UnexpectedCommand,
    UnexpectedVersion,
    UnexpectedService,
    UnsupportedUserAgent,
    Io,
    Deserialize(NetworkDeserializeError)
}
