use derive_more::{Constructor, From, Into};
use hdk3::prelude::*;

#[derive(Constructor, Serialize, Deserialize, SerializedBytes)]
pub struct Channel {
    name: String,
}

entry_def!(Channel EntryDef {
    id: "Channel".into(),
    ..Default::default()
});

#[derive(From, Into, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelList(Vec<Channel>);
