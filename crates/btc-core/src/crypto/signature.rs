use secp256k1::{Message, PublicKey, Secp256k1, ecdsa::Signature};

use crate::crypto::sha256;

pub fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let secp = Secp256k1::verification_only();

    let pk = match PublicKey::from_slice(public_key) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    let sig = match Signature::from_der(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    let digest = sha256(message);

    let msg = Message::from_digest(digest);

    secp.verify_ecdsa(msg, &sig, &pk).is_ok()
}
