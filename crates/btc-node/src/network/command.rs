use crate::network::NetworkError;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    Version,
    Verack,
    Ping,
    Pong,

    GetHeaders,
    Headers,

    GetData,
    Block,

    Inv,
    Tx,
}

impl Command {
    pub fn as_wire_name(&self) -> &'static [u8] {
        match self {
            Command::Version => b"version",
            Command::Verack => b"verack",
            Command::Ping => b"ping",
            Command::Pong => b"pong",
            Command::GetHeaders => b"getheaders",
            Command::Headers => b"headers",
            Command::GetData => b"getdata",
            Command::Inv => b"inv",
            Command::Tx => b"tx",
            Command::Block => b"block",
        }
    }


    /// Parse the 12-byte command field from the wire.
    /// The field is null-padded, e.g. b"version\0\0\0\0\0"
    pub fn from_wire_name(bytes: &[u8]) -> Option<Self> {
        // Strip trailing null bytes
        let name = bytes.split(|&b| b == 0).next().unwrap_or(bytes);

        match name {
            b"version" => Some(Command::Version),
            b"verack" => Some(Command::Verack),
            b"ping" => Some(Command::Ping),
            b"pong" => Some(Command::Pong),
            b"getheaders" => Some(Command::GetHeaders),
            b"headers" => Some(Command::Headers),
            b"getdata" => Some(Command::GetData),
            b"inv" => Some(Command::Inv),
            b"tx" => Some(Command::Tx),
            b"block" => Some(Command::Block),
            _ => None,
        }
    }
}

impl TryFrom<u8> for Command {
    type Error = NetworkError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Version),
            1 => Ok(Self::Verack),
            2 => Ok(Self::Ping),
            3 => Ok(Self::Pong),
            4 => Ok(Self::GetHeaders),
            5 => Ok(Self::Headers),
            6 => Ok(Self::GetData),
            7 => Ok(Self::Block),
            8 => Ok(Self::Inv),
            9 => Ok(Self::Tx),
            _ => Err(NetworkError::TypeCastFailed)
        }
    }
}