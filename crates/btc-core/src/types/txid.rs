#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TxId(pub [u8; 32]);

impl TxId {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}
