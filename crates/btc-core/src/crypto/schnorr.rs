use secp256k1::{
    Keypair, Secp256k1, SecretKey, XOnlyPublicKey, schnorr::Signature,
};

use crate::{crypto::sha256, taproot::{TaprootError, tweak::tweak_keypair}};

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


pub fn sign_tx_tr(data: &[u8], secret_key: &SecretKey, merkle_root: Option<[u8;32]>) -> Result<Signature, TaprootError> {
    let secp = Secp256k1::signing_only();

    let digest = sha256(data);

    let keypair = Keypair::from_secret_key(&secp, secret_key);
    let tweaked = tweak_keypair(&keypair, merkle_root)?;

   Ok(secp.sign_schnorr(&digest, &tweaked))
}


#[cfg(test)]
mod test {
    use crate::{crypto::generate_keypair_dummy, taproot::tweak_public_key};

use super::*;

    #[test]
    fn sign_then_verify(){
        
        let (sk, _pk) = generate_keypair_dummy();
        let secp = Secp256k1::new();

        let keypair =  Keypair::from_secret_key(&secp, &sk);
        let xonly_pk = tweak_public_key(&keypair.x_only_public_key().0, None).unwrap().0.serialize();
        let message = [1u8;32];
            
        let a = sign_tx_tr(&message, &sk, None).unwrap().to_byte_array();

        let a = verify_signature_tr(&xonly_pk, &message, &a);

        assert!(a)
    }
}