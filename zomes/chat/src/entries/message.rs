use crate::{error::ChatError, error::ChatResult, timestamp::Timestamp};
use hdk3::prelude::*;

use super::channel::Channel;

pub mod handlers;

/// The actual message data that is saved into the DHT
#[hdk_entry(id = "message")]
pub struct Message {
    pub uuid: String,
    pub content: String,
}


/// Input to the create message call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct MessageInput {
    /// By specifying the hash of the last message seen at the time of
    /// committing this message, it allows the app to order messages as best
    /// as it can. If this is the first message in the channel, this is None.
    pub last_message_seen: Option<EntryHash>,
    /// The channel which this message belongs to
    pub channel: Channel,
    /// The message to commit
    pub message: Message,
}

/// The message type that goes to the UI
#[derive(Serialize, Deserialize, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct MessageData {
    message: Message,
    entry_hash: EntryHash,
    created_by: AgentPubKey,
    created_at: Timestamp,
}

/// Input to the list messages call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListMessagesInput {
    channel: Channel,
    date: Date,
}

/// This is date you want to get messages for
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct Date {
    /// Year '2001'
    pub year: String,
    /// Month '01'
    pub month: String,
    /// Day '12'
    pub day: String,
}

/// The messages returned from list messages
#[derive(Serialize, Deserialize, SerializedBytes, derive_more::From)]
pub struct ListMessages {
    messages: Vec<MessageData>,
}

impl MessageData {
    pub fn new(header: Header, message: Message) -> ChatResult<Self> {
        let entry_hash = header
            .entry_hash()
            .ok_or(ChatError::WrongHeaderType)?
            .clone();
        Ok(Self {
            message,
            entry_hash,
            created_by: header.author().to_owned(),
            created_at: header.timestamp().to_owned(),
        })
    }
}

/// This key allows us to sort the messages by who they reply to
/// then by time
#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes, Ord, PartialOrd, Eq, PartialEq)]
struct LastSeenKey {
    parent_hash: EntryHash,
    timestamp: Option<Timestamp>,
}

impl LastSeenKey {
    pub fn new(parent_hash: EntryHash, timestamp: Timestamp) -> Self {
        Self {
            parent_hash,
            timestamp: Some(timestamp),
        }
    }
}

impl From<EntryHash> for LastSeenKey {
    fn from(parent_hash: EntryHash) -> Self {
        Self {
            parent_hash,
            timestamp: None,
        }
    }
}

impl From<LastSeenKey> for LinkTag {
    fn from(key: LastSeenKey) -> Self {
        Self::new(UnsafeBytes::from(
            SerializedBytes::try_from(key).expect("This serialization should never fail"),
        ))
    }
}

impl From<LinkTag> for LastSeenKey {
    fn from(t: LinkTag) -> Self {
        Self::try_from(SerializedBytes::from(UnsafeBytes::from(t.0)))
            .expect("This serialization should never fail")
    }
}
