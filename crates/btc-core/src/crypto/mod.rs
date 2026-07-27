pub mod hash;
pub mod signature;
pub mod schnorr;

pub use hash::{sha256d, sha256};
pub use signature::verify_signature;

// testing methods 
pub use signature::{generate_keypair_dummy, sign_tx};