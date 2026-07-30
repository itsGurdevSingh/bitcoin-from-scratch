use crate::transaction::Witness;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpendType {
    KeyPath,
    KeyPathAnnex(Vec<u8>),
    ScriptPath,
    ScriptPathAnnex(Vec<u8>),
}

impl SpendType {
    pub fn has_annex(&self) -> bool {
        match self {
            Self::KeyPathAnnex(_) | Self::ScriptPathAnnex(_) => true, 
            _ => false,
        }
    }

    pub fn get_annex_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::KeyPathAnnex(bytes) | Self::ScriptPathAnnex(bytes) => Some(bytes.clone()), 
            _ => None,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::KeyPath            => 0,
            Self::KeyPathAnnex(_)    => 1,
            Self::ScriptPath         => 2,
            Self::ScriptPathAnnex(_) => 3,
        }
    }
}

impl SpendType {
    pub fn get_spent_type(witness: &Witness) -> Option<Self> {
        if witness.stack.is_empty() {
            return None;
        }
        let witness_len = witness.stack.len();
        if witness_len == 1 &&
            witness.stack[0].len() == 65{ // 64 bytes of signature + 1 bytes of sig_hash type 
                return Some(Self::KeyPath);
        }

        if let Some(last) = witness.stack.last() {
            if last[0] == 0x50 {
                    return Some(Self::KeyPathAnnex(last.clone()));
            };
        };

        if witness_len > 2 {
            if witness.stack[witness_len - 3][0] == 0x50 {
                return Some(Self::ScriptPathAnnex(witness.stack[witness_len - 3].clone()));
            } else {
                return Some(Self::ScriptPath);
            }
        };
        None
    }
}