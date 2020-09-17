use crate::timestamp::Timestamp;
use hdk3::{hash_path::path::Component, prelude::*};
pub mod handlers;

/// The actual channel data that is saved into the DHT
#[hdk_entry(id = "channel_info")]
pub struct ChannelInfo {
    pub name: String,
    pub created_by: AgentPubKey,
    pub created_at: Timestamp,
}

/// Input to the create channel call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ChannelInput {
    name: String,
    channel: Channel,
}

#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes)]
pub struct Channel {
    category: String,
    id: String,
}

/// The message type that goes to the UI
#[derive(Serialize, Deserialize, SerializedBytes, derive_more::Constructor)]
#[serde(rename_all = "camelCase")]
pub struct ChannelData {
    pub channel: Channel,
    pub info: ChannelInfo,
}

/// Input to the list channels call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ListChannelsInput {
    category: String,
}

/// The channels returned from list messages
#[derive(Serialize, Deserialize, SerializedBytes, derive_more::From)]
pub struct ListChannels {
    channels: Vec<ChannelData>,
}

impl From<Channel> for Path {
    fn from(c: Channel) -> Self {
        let path = vec![Component::from(c.category), Component::from(c.id)];
        Path::from(path)
    }
}

impl TryFrom<&Path> for Channel {
    type Error = SerializedBytesError;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let path: &Vec<_> = p.as_ref();
        let channel = Channel {
            category: String::try_from(&path[0])?,
            id: String::try_from(&path[1])?,
        };
        Ok(channel)
    }
}

pub(crate) struct ChannelInfoTag;

impl ChannelInfoTag {
    const TAG: &'static [u8; 4] = b"info";
    pub(crate) fn tag() -> LinkTag {
        LinkTag::new(Self::TAG.clone())
    }
}
