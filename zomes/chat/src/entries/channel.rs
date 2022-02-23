use crate::timestamp::Timestamp;
use hdk::{hash_path::path::Component, prelude::*};
use uuid::Uuid;
pub mod handlers;
use std;

/// The actual channel data that is saved into the DHT
/// This is the actual name of the channel that
/// can change.
#[hdk_entry(id = "channel_info")]
#[derive(Clone, PartialEq, Eq)]
pub struct ChannelInfo {
    pub category: String,
    pub uuid: String,
    pub name: String,
    pub created_by: AgentPubKey,
    pub created_at: Timestamp,
}

/// Input to the create channel call
#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelInput {
    pub name: String,
    pub entry: Channel,
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
    pub entry: Channel,
    pub info: ChannelInfo,
}

/// Input to the list channels call
#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelListInput {
    pub category: String,
}

/// The channels returned from list channels
#[derive(Debug, Serialize, Deserialize, SerializedBytes, derive_more::From)]
pub struct ChannelList {
    pub channels: Vec<ChannelData>,
}

impl From<Channel> for Path {
    fn from(c: Channel) -> Self {
        let u = Uuid::parse_str(&c.uuid).unwrap();
        let path = vec![
            Component::from(c.category.as_bytes().to_vec()),
            Component::from(u.to_u128_le().to_le_bytes().to_vec()),
        ];
        Path::from(path)
    }
}

impl TryFrom<&Path> for Channel {
    type Error = SerializedBytesError;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let path: &Vec<_> = p.as_ref();
        let u128 = u128::from_le_bytes(path[1].as_ref().try_into().expect("wrong length"));
        let u = Uuid::from_u128(u128);
        let c: String = std::str::from_utf8(path[0].as_ref())
            .expect("bad string")
            .to_string();
        let channel = Channel {
            category: String::try_from(c)?,
            uuid: u.to_string(),
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
