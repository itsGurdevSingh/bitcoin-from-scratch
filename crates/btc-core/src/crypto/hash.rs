use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

pub fn sha256d(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);

    second.into()
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

pub fn hash160(data: &[u8]) -> [u8; 20] {
   let sha = Sha256::digest(data);

   let ripe = Ripemd160::digest(sha);

   ripe.into()
}

