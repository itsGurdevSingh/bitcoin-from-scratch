#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MerkleRoot(pub [u8; 32]);

impl MerkleRoot {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}