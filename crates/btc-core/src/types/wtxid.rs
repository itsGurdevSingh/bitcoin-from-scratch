#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WTxId(pub [u8; 32]);

impl WTxId {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl From<[u8; 32]> for WTxId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}