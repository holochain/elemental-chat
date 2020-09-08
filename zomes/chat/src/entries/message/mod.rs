use hdk3::prelude::*;
use crate::{timestamp::Timestamp};
use holo_hash::HasHash;

#[hdk_entry(id = "message")]
pub struct MessageEntry {
    uuid: String,
    content: String,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct MessageInput {
    base_entry_hash: EntryHash,
    message: MessageEntry,
}

#[derive(Serialize, Deserialize, SerializedBytes)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    uuid: String,
    content: String,
    entry_header_hash: HeaderHash,
    holochain_created_by: AgentPubKey,
    holochain_created_at: Timestamp,
}

impl TryFrom<Element> for Message {
    type Error = crate::Error;
    fn try_from(element: Element) -> Result<Self, Self::Error> {
        let message_entry: MessageEntry = element.entry().to_app_option()?.ok_or(Self::Error::Exists)?;
        Ok(Message { 
            uuid: message_entry.uuid,
            content: message_entry.content,
            entry_header_hash: element.header_hashed().as_hash().to_owned(),
            holochain_created_by: element.header().author().to_owned(),
            holochain_created_at: element.header().timestamp().to_owned(),
        })
    }
}

