use derive_more::{From, Into};
use hdk3::prelude::*;

mod channel;
mod channel_message;

pub use channel::*;
pub use channel_message::*;

#[derive(From, Into, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelName(String);

#[derive(From, Into, Serialize, Deserialize, SerializedBytes)]
pub struct StringContent(String);
