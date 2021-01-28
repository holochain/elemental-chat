use crate::timestamp::Timestamp;
use hdk3::{hash_path::path::Component, prelude::*};
pub mod handlers;

/// The actual channel data that is saved into the DHT
/// This is the actual name of the channel that
/// can change.
#[hdk_entry(id = "channel_info")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelInfo {
    pub name: String,
    pub created_by: AgentPubKey,
    pub created_at: Timestamp,
}

/// Input to the create channel call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ChannelInput {
    pub name: String,
    pub channel: Channel,
}

/// A channel is consists of the category it belongs to
/// and a unique id
#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes, PartialEq, Eq)]
pub struct Channel {
    pub category: String,
    pub uuid: String,
}

/*  using global chatters list for now.
impl Channel {
    pub fn chatters_path(&self) -> Path {
        let mut components: Vec<Component> = Path::from(self.clone()).into();
        components.push("chatters".into());
        components.into()
    }
}
 */

/// The message type that goes to the UI
#[derive(
    Serialize, Deserialize, SerializedBytes, derive_more::Constructor, Debug, Clone, PartialEq, Eq,
)]
#[serde(rename_all = "camelCase")]
pub struct ChannelData {
    pub channel: Channel,
    pub info: ChannelInfo,
    pub latest_chunk: u32,
}

/// Input to the list channels call
#[derive(Serialize, Deserialize, SerializedBytes)]
pub struct ChannelListInput {
    pub category: String,
}

/// The channels returned from list messages
#[derive(Debug, Serialize, Deserialize, SerializedBytes, derive_more::From)]
pub struct ChannelList {
    pub channels: Vec<ChannelData>,
}

impl From<Channel> for Path {
    fn from(c: Channel) -> Self {
        let path = vec![Component::from(c.category), Component::from(c.uuid)];
        Path::from(path)
    }
}

impl TryFrom<&Path> for Channel {
    type Error = SerializedBytesError;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let path: &Vec<_> = p.as_ref();
        let channel = Channel {
            category: String::try_from(&path[0])?,
            uuid: String::try_from(&path[1])?,
        };
        Ok(channel)
    }
}

/// A easy way to create the channel info tag
pub(crate) struct ChannelInfoTag;

impl ChannelInfoTag {
    const TAG: &'static [u8; 4] = b"info";

    /// Create the tag
    pub(crate) fn tag() -> LinkTag {
        LinkTag::new(*Self::TAG)
    }
}
