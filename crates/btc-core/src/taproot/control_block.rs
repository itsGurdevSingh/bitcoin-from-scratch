use secp256k1::XOnlyPublicKey;

use crate::{
    script::Script,
    serialization::{BitcoinDeserialize, BitcoinSerialize, DeserializeError},
    taproot::{TaprootError, tapbranch_hash, tapleaf_hash, tweak_public_key},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LeafVesrion {
    V1 = 0xC0,
}
impl LeafVesrion {
    fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0xC0 => Some(Self::V1),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlBlock {
    pub parity: bool,
    pub leaf_version: LeafVesrion,
    pub internal_key: XOnlyPublicKey,
    pub merkle_path: Vec<[u8; 32]>,
}

impl ControlBlock {

    pub fn new(script: &Script, all_scripts: &[&Script], leaf_version: LeafVesrion, xonly_public_key: &[u8]) -> Result<(Self, XOnlyPublicKey), TaprootError> {
        let merkle_root = Self::compute_root(all_scripts, leaf_version);

        let mut xonly_public_key_bytes = [0u8; 32];
        xonly_public_key_bytes.copy_from_slice(xonly_public_key);
        let xonly_pub_key = XOnlyPublicKey::from_byte_array(xonly_public_key_bytes).map_err(|_|TaprootError::InvalidXonlyBytes)?;
        
        let (xonly_pub_key_tweaked, parity) = tweak_public_key(&xonly_pub_key, Some(merkle_root))?;
        let proof = Self::build_proof(script, all_scripts, leaf_version)?;
        
        
        let control_block = ControlBlock {
            parity: parity as u8 > 0,
            leaf_version,
            internal_key: xonly_pub_key, // non tweaked
            merkle_path: proof,
        };
        
        Ok((control_block, xonly_pub_key_tweaked)) // control block used for sending and xonly_tweked key is present in pub script of utxo as unlocking script.
    }

    pub fn compute_root(scripts: &[&Script], leaf_version: LeafVesrion) -> [u8; 32] {
        let mut current_level = Vec::new();
        for script in scripts {
            current_level.push(tapleaf_hash(leaf_version, *script));
        }
        current_level.sort();

        while current_level.len() > 1 {
            let mut idx: usize = 0;
            let mut next_level = Vec::new();

            while current_level.len() > idx {
                next_level.push(match current_level.len() > idx + 1 {
                    true => tapbranch_hash(current_level[idx], current_level[idx + 1]),
                    false => tapbranch_hash(current_level[idx], current_level[idx]),
                });
                idx += 2;
            }
            current_level = next_level;
        }
        current_level[0]
    }

    pub fn build_proof(
        script: &Script,
        all_scripts: &[&Script],
        leaf_version: LeafVesrion,
    ) -> Result<Vec<[u8; 32]>, TaprootError> {
        let mut target = tapleaf_hash(leaf_version, script);
        let mut proof: Vec<[u8; 32]> = Vec::new();

        let mut has_target: bool = false;
        let mut current_level = Vec::new();
        for script in all_scripts {
            let leaf = tapleaf_hash(LeafVesrion::V1, *script);
            current_level.push(leaf);

            if leaf == target {
                has_target = true
            }
        }

        if !has_target {
            Err(TaprootError::TargetScriptNotExist)?;
        }

        current_level.sort();

        while current_level.len() > 1 {

            let mut idx: usize = 0;
            let mut next_level = Vec::new();

            while current_level.len() > idx {
                next_level.push(match current_level.len() > idx + 1 {
                    true => {
                        let branch = tapbranch_hash(current_level[idx], current_level[idx + 1]);
                        if current_level[idx] == target {
                            proof.push(current_level[idx + 1]);
                            target = branch
                        };
                        if current_level[idx + 1] == target {
                            proof.push(current_level[idx]);
                            target = branch
                        };
                        branch
                    }
                    false => {
                        let branch = tapbranch_hash(current_level[idx], current_level[idx]);
                        if current_level[idx] == target {
                            proof.push(current_level[idx]);
                            target = branch;
                        };
                        branch
                    }
                });
                idx += 2;
            }
            current_level = next_level;
        }

        Ok(proof)
    }

    pub fn verify_proof(
        script: &Script,
        control_block: &ControlBlock,
        xonly_public_key: &[u8],
    ) -> bool {
        let script_leaf_hash = tapleaf_hash(control_block.leaf_version, script);

        let mut current = script_leaf_hash;

        for path in control_block.merkle_path.iter() {
            current = tapbranch_hash(current, path.clone())
        }

        let (computed_output_key, parity) =
            match tweak_public_key(&control_block.internal_key, Some(current)) {
                Ok(data) => data,
                Err(_) => {
                    return false;
                }
            };

        if (parity.to_u8() > 0) != control_block.parity {
            return false;
        };

        if &computed_output_key.serialize() != xonly_public_key {
            return false;
        };

        true
    }
}

impl BitcoinSerialize for ControlBlock {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let control_byte = self.leaf_version as u8 | (self.parity as u8);
        bytes.push(control_byte);
        bytes.extend(self.internal_key.serialize());

        for path in self.merkle_path.iter() {
            bytes.extend_from_slice(path);
        }

        bytes
    }
}

impl BitcoinDeserialize for ControlBlock {
    type Error = DeserializeError;
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize), Self::Error> {
        if bytes.len() < 33 {
            Err(DeserializeError::UnexpectedEndOfBytes)?;
        }
        if (bytes.len() - 33) % 32 != 0 {
            Err(DeserializeError::UnexpectedEndOfBytes)?
        }

        let mut consumed: usize = 0;
        let parity = (bytes[consumed] & 1) == 1;

        let leaf_version = LeafVesrion::from_u8(bytes[consumed] & 0xFE)
            .ok_or(DeserializeError::UnexpectedEndOfBytes)?;
        consumed += 1;
        let mut internal_key_bytes = [0u8; 32];

        internal_key_bytes.copy_from_slice(&bytes[consumed..consumed + 32]);
        consumed += 32;

        let internal_key = XOnlyPublicKey::from_byte_array(internal_key_bytes)
            .map_err(|_| DeserializeError::UnexpectedEndOfBytes)?;

        let mut merkle_path = Vec::new();

        for _ in 0..((bytes.len() - consumed) / 32) {
            let mut a = [0u8; 32];
            a.copy_from_slice(&bytes[consumed..consumed + 32]);
            consumed += 32;
            merkle_path.push(a);
        }
        Ok((
            Self {
                parity,
                leaf_version,
                internal_key,
                merkle_path,
            },
            consumed,
        ))
    }
}

#[cfg(test)]
mod test {

    use std::assert_eq;

    use secp256k1::{Keypair, Secp256k1};

    use crate::{
        crypto::generate_keypair_dummy, script::{OpCode, ScriptItem},
    };

    use super::*;

    #[test]
    fn serlize_then_deselize_result_same() {
        let control_block = ControlBlock {
            parity: true,
            leaf_version: LeafVesrion::V1,
            internal_key: XOnlyPublicKey::from_byte_array([1u8; 32]).unwrap(),
            merkle_path: vec![[0u8; 32], [1u8; 32], [2u8; 32]],
        };

        let ser_block = control_block.serialize();

        let (der_block, _) = ControlBlock::deserialize(&ser_block).unwrap();

        assert_eq!(control_block, der_block);
    }

    #[test]
    fn compute_root_build_proof_then_verify_proof() {
        let script_a = Script {
            items: vec![ScriptItem::Op(OpCode::Op1)],
        };
        let script_b = Script {
            items: vec![ScriptItem::Op(OpCode::Op2)],
        };
        let script_c = Script {
            items: vec![ScriptItem::Op(OpCode::Op3)],
        };
        let script_d = Script {
            items: vec![ScriptItem::Op(OpCode::Op4)],
        };
        let script_e = Script {
            items: vec![ScriptItem::Op(OpCode::Op5)],
        };
        let script_f = Script {
            items: vec![ScriptItem::Op(OpCode::Op6)],
        };

        let scripts = [&script_a, &script_b, &script_c, &script_d, &script_e, &script_f];
        let leaf_version = LeafVesrion::V1;

        let merkle_root = ControlBlock::compute_root(&scripts, leaf_version);

        let (sk, _pk) = generate_keypair_dummy();
        let secp = Secp256k1::new();
        let keypair = Keypair::from_secret_key(&secp, &sk);
        let (xonly_pub_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        let (xonly_public_key, parity) =
            tweak_public_key(&xonly_pub_key, Some(merkle_root)).unwrap();

        let proof = ControlBlock::build_proof(&script_a, &scripts, leaf_version).unwrap();

        let control_block = ControlBlock {
            parity: parity as u8 > 0,
            leaf_version,
            internal_key: xonly_pub_key, // non tweaked
            merkle_path: proof,
        };

        let verify_proof =
            ControlBlock::verify_proof(&script_a, &control_block, &xonly_public_key.serialize());

        assert!(verify_proof)
    }

    #[test]
    fn build_block_then_verify() {
         let script_a = Script {
            items: vec![ScriptItem::Op(OpCode::Op1)],
        };
        let script_b = Script {
            items: vec![ScriptItem::Op(OpCode::Op2)],
        };
        let script_c = Script {
            items: vec![ScriptItem::Op(OpCode::Op3)],
        };
        let script_d = Script {
            items: vec![ScriptItem::Op(OpCode::Op4)],
        };
        let script_e = Script {
            items: vec![ScriptItem::Op(OpCode::Op5)],
        };
        let script_f = Script {
            items: vec![ScriptItem::Op(OpCode::Op6)],
        };

        let scripts = [&script_a, &script_b, &script_c, &script_d, &script_e, &script_f];
        let leaf_version = LeafVesrion::V1;

        //======================GENERATE KEY PAIR ======================//
        let (sk, _pk) = generate_keypair_dummy();
        let secp = Secp256k1::new();
        let keypair = Keypair::from_secret_key(&secp, &sk);
        let (xonly_pub_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        //=====================NEW CONTROL BLOCK =============================//
        let (control_block, xonly_public_key_tweaked) = ControlBlock::new(&script_b, &scripts, leaf_version, &xonly_pub_key.serialize()).unwrap();

        assert!(ControlBlock::verify_proof(&script_b, &control_block, &xonly_public_key_tweaked.serialize()));

    }
}
