use crate::{block::constants::WITNESS_COMMITMENT_HEADER, script::{OpCode, Script, ScriptItem}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptType {
    P2PKH,
    P2SH,
    P2WPKH,
    P2WSH,
    None
}

impl ScriptType {


    pub fn is_type_of(script_pub: &Script, script_sig: &Script) -> Self {

        if ScriptType::is_p2sh_script(&script_pub, &script_sig) {
            return Self::P2SH;
        };
        if ScriptType::is_p2wsh_script(&script_pub, &script_sig) {
            return Self::P2WSH;
        };
        if ScriptType::is_p2wpkh_script(&script_pub, &script_sig) {
            return Self::P2WPKH;
        };
        if ScriptType::is_p2pkh_script(&script_pub, &script_sig) {
            return Self::P2PKH;
        };
        Self::None
    }


    fn is_p2sh_script(script_pub: &Script, script_sig: &Script) -> bool {
        if !ScriptType::is_push_only(script_sig) {
            return false;
        }
        match script_pub.items.as_slice() {
            [
                ScriptItem::Op(OpCode::Hash160),
                ScriptItem::PushData(data),
                ScriptItem::Op(OpCode::Equal),
            ] => data.len() == 20,

            _ => false,
        }
    }

    fn is_p2wsh_script(script_pub: &Script, script_sig: &Script) -> bool {
        if !ScriptType::is_push_only(script_sig) {
            return false;
        }
        match script_pub.items.as_slice() {
            [ScriptItem::Op(OpCode::Op0), ScriptItem::PushData(data)] => data.len() == 32,

            _ => false,
        }
    }
    fn is_p2wpkh_script(script_pub: &Script, script_sig: &Script) -> bool {
        if !ScriptType::is_push_only(script_sig) {
            return false;
        }
        match script_pub.items.as_slice() {
            [ScriptItem::Op(OpCode::Op0), ScriptItem::PushData(data)] => data.len() == 20,

            _ => false,
        }
    }
    fn is_p2pkh_script(script_pub: &Script, script_sig: &Script) -> bool {
        if !ScriptType::is_push_only(script_sig) {
            return false;
        }
        match script_pub.items.as_slice() {
            [ScriptItem::Op(OpCode::Dup),
             ScriptItem::Op(OpCode::Hash160),
             ScriptItem::PushData(data),
             ScriptItem::Op(OpCode::EqualVerify),
             ScriptItem::Op(OpCode::CheckSig)
             ] => data.len() == 20,

            _ => false,
        }
    }

    

    fn is_push_only(script: &Script) -> bool {
        script.items.iter().all(|item| match item {
            ScriptItem::PushData(_) => true,
            ScriptItem::Op(code) => code.is_push_only(),
        })
    }


    pub fn is_witness_commitment_script(script: &Script, commitment_hash: &Vec<u8>) -> bool {
        let commitment_header = &WITNESS_COMMITMENT_HEADER.to_vec();
        match script.items.as_slice() {
            [ScriptItem::Op(OpCode::Return),
             ScriptItem::PushData(header),
             ScriptItem::PushData(data),
             ] => {
                header == commitment_header && data == commitment_hash
             },

             _=> false
        }
}
}