use crate::{error::ChatError, error::ChatResult, timestamp::Timestamp};
use hdk3::prelude::*;

use super::channel::{Channel, ChannelData};

pub mod handlers;

/// The actual message data that is saved into the DHT
#[hdk_entry(id = "message")]
#[derive(Clone, Debug)]
pub struct Message {
    pub uuid: String,
    pub content: String,
}

/// This allows the app to properly order messages.
/// This message is either the first message of the time block
/// or has another message that was observed at the time of sending.
#[derive(Serialize, Deserialize, SerializedBytes)]
pub enum LastSeen {
    First,
    Message(EntryHash),
}

/// Input to the create message call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct MessageInput {
    pub last_seen: LastSeen,
    pub channel: Channel,
    pub message: Message,
    pub chunk: u32,
}

/// The message type that goes to the UI
#[derive(Serialize, Deserialize, Clone, Debug, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct MessageData {
    message: Message,
    entry_hash: EntryHash,
    created_by: AgentPubKey,
    created_at: Timestamp,
}

// Input to the signal_specific_chatters call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct SignalSpecificInput {
    signal_message_data: SignalMessageData,
    chatters: Vec<AgentPubKey>
}


/// The message type that goes to the UI via emit_signal
#[derive(Serialize, Deserialize, SerializedBytes, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessageData {
    pub message_data: MessageData,
    pub channel_data: ChannelData,
}

/// Input to the list messages call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListMessagesInput {
    channel: Channel,
    chunk: Chunk,
    active_chatter: bool,
}
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct Chunk {
    start: u32,
    end: u32,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct SigResults {
    pub total: usize,
    pub sent: Vec<String>,
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
