use derive_more::{Constructor, From, Into};
use hdk3::prelude::*;

#[derive(Constructor, Serialize, Deserialize, SerializedBytes)]
pub struct Channel {
    name: String,
}

impl Channel {
    pub fn entry_def() -> EntryDef {
        EntryDef {
            id: "Channel".into(),
            crdt_type: CrdtType,
            required_validations: RequiredValidations::default(),
            visibility: EntryVisibility::Public,
        }
    }
}

impl From<&Channel> for EntryDefId {
    fn from(_: &Channel) -> Self {
        Channel::entry_def().id
    }
}

#[derive(From, Into, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelList(Vec<Channel>);
