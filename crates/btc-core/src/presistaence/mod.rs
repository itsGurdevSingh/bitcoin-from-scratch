pub mod db_persistence;
pub mod dummy_impl;
pub mod error;

pub use db_persistence::DbPersistence;
pub use dummy_impl::Store;
pub use error::PersistenceError;