use crate::taproot::tagged_hash;

pub fn tapbranch_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut bytes: Vec<u8> = Vec::new();

    if left > right {
        bytes.extend(right);
        bytes.extend(left);
    } else {
        bytes.extend(left);
        bytes.extend(right);
    }

    tagged_hash("TapBranch", &bytes)
}
