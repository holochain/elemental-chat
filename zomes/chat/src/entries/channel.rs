use hdk3::prelude::*;
use crate::{timestamp::Timestamp};
pub mod handlers;

#[hdk_entry(id = "channel")]
pub struct ChannelEntry {
    pub uuid: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ChannelInput {
    path: String,
    channel: ChannelEntry,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListChannelsInput {
    path: String,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListChannels{
    channels: Vec<Channel>,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    uuid: String,
    content: String,
    hash_entry: EntryHash,
    holochain_created_by: AgentPubKey,
    holochain_created_at: Timestamp,
}

impl Channel {
    pub fn new(header: Header, channel_entry: ChannelEntry, hash_entry: EntryHash) -> Self {
        Channel { 
            uuid: channel_entry.uuid,
            content: channel_entry.content,
            hash_entry,
            holochain_created_by: header.author().to_owned(),
            holochain_created_at: header.timestamp().to_owned(),
        }
    }
}

