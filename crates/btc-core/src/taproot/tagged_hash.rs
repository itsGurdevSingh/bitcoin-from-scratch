use crate::crypto::sha256;

pub fn tagged_hash(tag: &str, data: &[u8]) -> [u8; 32] {
    let tag_hash = sha256(tag.as_bytes());

    let mut bytes = Vec::new();

    bytes.extend_from_slice(&tag_hash);
    bytes.extend_from_slice(&tag_hash);
    bytes.extend(data);

    sha256(&bytes)
}
