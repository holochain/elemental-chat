mod entries;

use entries::message::{MessageEntry};
use entries::channel::{ChannelEntry};
use hdk3::prelude::Path;
use hdk3::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Serialization(#[from] SerializedBytesError),
    #[error("Does not exist")]
    Exists,
}

entry_defs![
    Path::entry_def(), 
    MessageEntry::entry_def(), 
    ChannelEntry::entry_def()
];
