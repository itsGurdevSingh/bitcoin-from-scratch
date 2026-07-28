use crate::{script::Script, serialization::{BitcoinSerialize, compact_size::get_compact_size}, taproot::{control_block::LeafVesrion, tagged_hash}};

pub fn tapleaf_hash(
    leaf_version: LeafVesrion,
    script: &Script,
) -> [u8;32] {

    let script_bytes = script.serialize();
    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(leaf_version as u8);
    bytes.extend(get_compact_size(script_bytes.len()));
    bytes.extend(script_bytes);

    tagged_hash("TapLeaf", &bytes)
}