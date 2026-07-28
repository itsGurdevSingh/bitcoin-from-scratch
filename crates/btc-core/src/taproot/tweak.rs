use secp256k1::{Keypair, Parity, Scalar, Secp256k1, XOnlyPublicKey};

use crate::taproot::{TaprootError, tagged_hash};

pub fn tap_tweak_hash(
    internal_key: &XOnlyPublicKey,
    merkle_root: Option<[u8;32]>,
) -> [u8;32] {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&internal_key.serialize());

    if let Some(root) = merkle_root {
        bytes.extend_from_slice(&root);
    }

    tagged_hash("TapTweak", &bytes)
}


pub fn tweak_public_key(
    internal_key: &XOnlyPublicKey,
    merkle_root: Option<[u8; 32]>,
) -> Result<(XOnlyPublicKey, Parity), TaprootError> {
    let tweak_hash = tap_tweak_hash(internal_key, merkle_root);

    let tweak = Scalar::from_be_bytes(tweak_hash)
        .map_err(|_| TaprootError::InvalidTweak)?;

    let secp = Secp256k1::verification_only();

    let (output_key, parity) =
        internal_key.add_tweak(&secp, &tweak)
            .map_err(|_| TaprootError::InvalidTweak)?;

    Ok((output_key, parity))
}


// WALLET method for testing purpose (not belong to core)
pub fn tweak_keypair(
    keypair: &Keypair,
    merkle_root: Option<[u8; 32]>,
) -> Result<Keypair, TaprootError> {
    let (internal_key, _) = keypair.x_only_public_key();

    let tweak_hash = tap_tweak_hash(&internal_key, merkle_root);

    let tweak = Scalar::from_be_bytes(tweak_hash)
        .map_err(|_| TaprootError::InvalidTweak)?;

    let secp = Secp256k1::new();

    let tweaked = keypair
        .add_xonly_tweak(&secp, &tweak)
        .map_err(|_| TaprootError::InvalidTweak)?;

    Ok(tweaked)
}