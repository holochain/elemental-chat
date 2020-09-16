use crate::timestamp::Timestamp;
use hdk3::prelude::*;
pub mod handlers;

/// The actual channel data that is saved into the DHT
#[hdk_entry(id = "channel")]
pub struct ChannelEntry {
    pub uuid: String,
    pub content: String,
}

/// Input to the create channel call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ChannelInput {
    path: String,
    channel: ChannelEntry,
}

/// The message type that goes to the UI
#[derive(Serialize, Deserialize, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    uuid: String,
    content: String,
    entry_hash: EntryHash,
    created_by: AgentPubKey,
    created_at: Timestamp,
}

/// Input to the list channels call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListChannelsInput {
    path: String,
}

/// The channels returned from list messages
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListChannels {
    channels: Vec<Channel>,
}

impl Channel {
    pub fn new(header: Header, channel_entry: ChannelEntry, entry_hash: EntryHash) -> Self {
        Channel {
            uuid: channel_entry.uuid,
            content: channel_entry.content,
            entry_hash,
            created_by: header.author().to_owned(),
            created_at: header.timestamp().to_owned(),
        }
    }
}
