use hdk3::prelude::*;
use std::convert::Infallible;

#[derive(thiserror::Error, Debug)]
pub enum ChatError {
    #[error(transparent)]
    Serialization(#[from] SerializedBytesError),
    #[error(transparent)]
    Infallible(#[from] Infallible),
    #[error(transparent)]
    EntryError(#[from] EntryError),
    #[error("Failed to convert an agent link tag to an agent pub key")]
    AgentTag,
    #[error(transparent)]
    Wasm(#[from] WasmError),
    #[error("Header that was just committed is missing. This means something went really wrong")]
    MissingLocalHeader,
    #[error("Tried to use a header without an entry as for where it only makes sense to use a new entry header")]
    WrongHeaderType,
    #[error("Channel at path {0} doesn't exist")]
    MissingChannel(String),
    #[error("Something is fatally wrong with this app\n Please post a bug report on the repo\n Error: {0}")]
    DataFormatError(&'static str),
}

pub type ChatResult<T> = Result<T, ChatError>;

impl From<ChatError> for WasmError {
    fn from(c: ChatError) -> Self {
        WasmError::Zome(c.to_string())
    }
}
