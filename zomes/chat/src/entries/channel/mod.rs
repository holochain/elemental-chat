use hdk3::prelude::*;
use crate::{timestamp::Timestamp};
use holo_hash::HasHash;

#[hdk_entry(id = "channel")]
pub struct ChannelEntry {
    uuid: String,
    content: String,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ChannelInput {
    path: String,
    channel: ChannelEntry,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    uuid: String,
    content: String,
    entry_header_hash: HeaderHash,
    holochain_created_by: AgentPubKey,
    holochain_created_at: Timestamp,
}

impl TryFrom<Element> for Channel {
    type Error = crate::Error;
    fn try_from(element: Element) -> Result<Self, Self::Error> {
        let channel_entry: ChannelEntry = element.entry().to_app_option()?.ok_or(Self::Error::Exists)?;
        Ok(Channel { 
            uuid: channel_entry.uuid,
            content: channel_entry.content,
            entry_header_hash: element.header_hashed().as_hash().to_owned(),
            holochain_created_by: element.header().author().to_owned(),
            holochain_created_at: element.header().timestamp().to_owned(),
        })
    }
}

