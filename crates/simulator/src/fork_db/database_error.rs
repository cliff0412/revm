
use database_interface::DBErrorMarker;
use futures::channel::mpsc::{SendError, TrySendError};
use primitives::{Address, B256, StorageKey};
use std::sync::{mpsc::RecvError, Arc};
use thiserror::Error;

// Errors that can happen when working with [`revm::Database`]
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Failed to fetch AccountInfo {0:?}")]
    MissingAccount(Address),
    #[error("Could should already be loaded: {0:?}")]
    MissingCode(B256),
    #[error(transparent)]
    Recv(#[from] RecvError),
    #[error(transparent)]
    Send(#[from] SendError),
    #[error("{0}")]
    Message(String),
    #[error("Failed to get account for {0:?}: {0:?}")]
    GetAccount(Address, Arc<eyre::Error>),
    #[error("Failed to get storage for {0:?} at {1:?}: {2:?}")]
    GetStorage(Address, StorageKey, Arc<eyre::Error>),
    #[error("Failed to get block hash for {0}: {1:?}")]
    GetBlockHash(u64, Arc<eyre::Error>),
}

impl DBErrorMarker for DatabaseError {}

impl<T> From<TrySendError<T>> for DatabaseError {
    fn from(err: TrySendError<T>) -> Self {
        err.into_send_error().into()
    }
}

impl DatabaseError {
    // Create a new error with a message
    pub fn msg(msg: impl Into<String>) -> Self {
        DatabaseError::Message(msg.into())
    }
}

// Result alias with `DatabaseError` as error
pub type DatabaseResult<T> = Result<T, DatabaseError>;
