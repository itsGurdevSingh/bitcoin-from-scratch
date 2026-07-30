use crate::taproot::tagged_hash;

pub fn taproot_sighash(sigmsg: &[u8]) -> [u8;32] {
    let mut bytes = Vec::new();

    bytes.push(0);          // epoch

    bytes.extend(sigmsg);

    tagged_hash("TapSighash", &bytes)
}