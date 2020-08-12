use super::StringContent;
use derive_more::{Constructor, From, Into};
use hdk3::prelude::*;

#[derive(Constructor, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelMessage {
    message: String,
}

entry_def!(ChannelMessage EntryDef {
    id: "ChannelMessage".into(),
    ..Default::default()
});

impl From<StringContent> for ChannelMessage {
    fn from(content: StringContent) -> Self {
        ChannelMessage { message: content.0 }
    }
}

#[derive(From, Into, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelMessageList(Vec<ChannelMessage>);
