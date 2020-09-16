use crate::timestamp::Timestamp;
use hdk3::prelude::*;

pub mod handlers;

/// The actual message data that is saved into the DHT
#[hdk_entry(id = "message")]
pub struct MessageEntry {
    pub uuid: String,
    pub content: String,
}

/// Who this message replies to.
/// Either the first message so the channel
/// or another parent message.
#[derive(Serialize, Deserialize, SerializedBytes)]
pub enum ReplyTo {
    Channel,
    Message(EntryHash),
}

/// Input to the create message call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct MessageInput {
    pub reply_to: ReplyTo,
    pub channel_entry_hash: EntryHash,
    pub message: MessageEntry,
}

/// The message type that goes to the UI
#[derive(Serialize, Deserialize, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    message: MessageEntry,
    entry_hash: EntryHash,
    created_by: AgentPubKey,
    created_at: Timestamp,
}

/// Input to the list messages call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListMessagesInput {
    channel_entry_hash: EntryHash,
}

/// The messages returned from list messages
#[derive(Serialize, Deserialize, SerializedBytes, derive_more::From)]
pub struct ListMessages {
    messages: Vec<Message>,
}

impl Message {
    pub fn new(header: Header, message: MessageEntry, entry_hash: EntryHash) -> Self {
        Message {
            message,
            entry_hash,
            created_by: header.author().to_owned(),
            created_at: header.timestamp().to_owned(),
        }
    }
}

/// This key allows us to sort the messages by who they reply to
/// then by time
#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes, Ord, PartialOrd, Eq, PartialEq)]
struct ReplyKey {
    reply_to_hash: EntryHash,
    timestamp: Option<Timestamp>,
}

impl ReplyKey {
    pub fn new(reply_to_hash: EntryHash, timestamp: Timestamp) -> Self {
        Self {
            reply_to_hash,
            timestamp: Some(timestamp),
        }
    }
}

impl From<EntryHash> for ReplyKey {
    fn from(reply_to_hash: EntryHash) -> Self {
        Self {
            reply_to_hash,
            timestamp: None,
        }
    }
}

impl From<ReplyKey> for LinkTag {
    fn from(key: ReplyKey) -> Self {
        Self::new(UnsafeBytes::from(
            SerializedBytes::try_from(key).expect("This serialization should never fail"),
        ))
    }
}

impl From<LinkTag> for ReplyKey {
    fn from(t: LinkTag) -> Self {
        Self::try_from(SerializedBytes::from(UnsafeBytes::from(t.0)))
            .expect("This serialization should never fail")
    }
}
