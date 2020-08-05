use derive_more::{From, Into};
use hdk3::prelude::*;

mod channel;
mod channel_message;

pub use channel::*;
pub use channel_message::*;

#[derive(Debug, From, Into, Serialize, Deserialize, SerializedBytes)]
pub struct ChannelName(String);

#[derive(Debug, From, Into, Serialize, Deserialize, SerializedBytes)]
pub struct StringContent(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ser() {
        dbg!(SerializedBytes::try_from(ChannelName("hello".into())).unwrap());
        
    }
}