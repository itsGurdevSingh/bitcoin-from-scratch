use secp256k1::{Message, PublicKey, Secp256k1, SecretKey, ecdsa::Signature};

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

pub fn generate_keypair_dummy() -> (SecretKey, PublicKey){
    let secp = Secp256k1::new();

    let secret_key = SecretKey::from_byte_array([1u8; 32]).unwrap();

    let public_key =
    PublicKey::from_secret_key(
        &secp,
        &secret_key,
    );

    (secret_key, public_key)

}

pub fn sign_tx(data: &[u8], secret_key: &SecretKey) -> Signature {
    let secp = Secp256k1::signing_only();

    let digest = sha256(data);

    let msg = Message::from_digest(digest);

    secp.sign_ecdsa(msg, secret_key)
}
