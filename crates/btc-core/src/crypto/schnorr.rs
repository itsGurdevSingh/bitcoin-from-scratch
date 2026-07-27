use secp256k1::{
    Keypair, Secp256k1, SecretKey, XOnlyPublicKey, schnorr::Signature,
};

use crate::crypto::sha256;

pub fn verify_signature_tr(xonly_public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let secp = Secp256k1::verification_only();

    let pk_bytes: [u8; 32] = match xonly_public_key.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let pk = match XOnlyPublicKey::from_byte_array(pk_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let sg_bytes: [u8; 64] = match signature.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let sig = Signature::from_byte_array(sg_bytes);

    let digest = sha256(message);

    secp.verify_schnorr(&sig, &digest, &pk).is_ok()
}


pub fn sign_tx_tr(data: &[u8], secret_key: &SecretKey) -> Signature {
    let secp = Secp256k1::signing_only();

    let digest = sha256(data);

    let keypair = Keypair::from_secret_key(&secp, secret_key);

    secp.sign_schnorr(&digest, &keypair)
}
