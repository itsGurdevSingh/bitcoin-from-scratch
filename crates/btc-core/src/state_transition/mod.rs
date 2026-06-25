pub mod state_transition;
pub mod processor;
pub mod error;

pub use state_transition::{StateTransition, SpentUtxo, CreatedUtxo};
pub use processor::TransactionProcessor;
pub use error::ProcessorError;
