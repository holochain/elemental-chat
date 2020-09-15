use crate::timestamp::Timestamp;
use hdk3::prelude::*;

pub mod handlers;

#[hdk_entry(id = "message")]
pub struct MessageEntry {
    pub uuid: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub enum ReplyTo {
    Channel(EntryHash),
    Message(EntryHash),
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct MessageInput {
    pub reply_to: ReplyTo,
    pub message: MessageEntry,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListMessagesInput {
    channel_entry_hash: EntryHash,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListMessages {
    messages: Vec<Message>,
}
#[derive(Serialize, Deserialize, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    uuid: String,
    content: String,
    entry_hash: EntryHash,
    holochain_created_by: AgentPubKey,
    holochain_created_at: Timestamp,
}

impl Message {
    pub fn new(header: Header, message_entry: MessageEntry, entry_hash: EntryHash) -> Self {
        Message {
            uuid: message_entry.uuid,
            content: message_entry.content,
            entry_hash,
            holochain_created_by: header.author().to_owned(),
            holochain_created_at: header.timestamp().to_owned(),
        }
    }
}
