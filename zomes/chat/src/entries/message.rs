use crate::{error::ChatError, error::ChatResult, timestamp::Timestamp};
use hdk::prelude::*;

use super::channel::{Channel, ChannelData};
pub mod handlers;

/// The actual message data that is saved into the DHT
#[hdk_entry(id = "message")]
#[derive(Clone, PartialEq)]
pub struct Message {
    pub uuid: String,
    pub content: String,
}

/// This allows the app to properly order messages.
/// This message is either the first message of the time block
/// or has another message that was observed at the time of sending.
#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
pub enum LastSeen {
    First,
    Message(EntryHash),
}

/// Input to the create message call
#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
pub struct MessageInput {
    pub last_seen: LastSeen,
    pub channel: Channel,
    pub entry: Message,
    pub chunk: u32,
}

/// The message type that goes to the UI
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct MessageData {
    pub entry: Message,
    pub entry_hash: EntryHash,
    pub created_by: AgentPubKey,
    pub created_at: Timestamp,
}

// Input to the signal_specific_chatters call
#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct SignalSpecificInput {
    signal_message_data: SignalMessageData,
    chatters: Vec<AgentPubKey>,
    include_active_chatters: Option<bool>,
}

/// The message type that goes to the UI via emit_signal
#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessageData {
    pub message_data: MessageData,
    pub channel_data: ChannelData,
}

/// Input to the list messages call
#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes)]
pub struct ListMessagesInput {
    pub channel: Channel,
    pub chunk: Chunk,
    pub active_chatter: bool,
}
/// Input to the list messages call
#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes)]
pub struct ListMessagesBatchInput {
    pub channel: Channel,
    pub earlier_than: Timestamp,
    // Keep expanding search interval until this count is reached
    pub target_message_count: usize, // UI will say 20 to start
    pub active_chatter: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes)]
pub struct Chunk {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct SigResults {
    pub total: usize,
    pub sent: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct ActiveChatters {
    pub chatters: Vec<AgentPubKey>,
}

/// The messages returned from list messages
#[derive(Debug, Serialize, Deserialize, SerializedBytes, derive_more::From, Clone, PartialEq)]
pub struct ListMessages {
    pub messages: Vec<MessageData>,
}

impl MessageData {
    pub fn new(header: Header, message: Message) -> ChatResult<Self> {
        let entry_hash = header
            .entry_hash()
            .ok_or(ChatError::WrongHeaderType)?
            .clone();
        Ok(Self {
            entry: message,
            entry_hash,
            created_by: header.author().to_owned(),
            created_at: header.timestamp().to_owned(),
        })
    }
}

impl SignalMessageData {
    pub fn new(message_data: MessageData, channel_data: ChannelData) -> Self {
        Self {
            message_data,
            channel_data,
        }
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
