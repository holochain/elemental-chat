use super::StringContent;
use derive_more::{Constructor, From, Into};
use hdk3::prelude::*;

#[derive(Constructor, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelMessage {
    message: String,
}

impl ChannelMessage {
    pub fn entry_def() -> EntryDef {
        EntryDef {
            id: "ChannelMessage".into(),
            crdt_type: CrdtType,
            required_validations: RequiredValidations::default(),
            visibility: EntryVisibility::Public,
        }
    }
}

impl From<&ChannelMessage> for EntryDefId {
    fn from(_: &ChannelMessage) -> Self {
        ChannelMessage::entry_def().id
    }
}

impl From<StringContent> for ChannelMessage {
    fn from(content: StringContent) -> Self {
        ChannelMessage { message: content.0 }
    }
}

#[derive(From, Into, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelMessageList(Vec<ChannelMessage>);
