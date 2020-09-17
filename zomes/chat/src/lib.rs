use channel::{ChannelData, ChannelInfo, ChannelInput, ListChannels, ListChannelsInput};
use entries::{channel, message};
use error::ChatResult;
use hdk3::prelude::Path;
use hdk3::prelude::*;
use message::{ListMessages, ListMessagesInput, Message, MessageData, MessageInput};

mod entries;
mod error;
mod utils;

entry_defs![
    Path::entry_def(),
    Message::entry_def(),
    ChannelInfo::entry_def()
];

#[hdk_extern]
fn create_channel(channel_input: ChannelInput) -> ChatResult<ChannelData> {
    channel::handlers::create_channel(channel_input)
}

#[hdk_extern]
fn create_message(message_input: MessageInput) -> ChatResult<MessageData> {
    message::handlers::create_message(message_input)
}

#[hdk_extern]
fn list_channels(list_channels_input: ListChannelsInput) -> ChatResult<ListChannels> {
    channel::handlers::list_channels(list_channels_input)
}

#[hdk_extern]
fn list_messages(list_messages_input: ListMessagesInput) -> ChatResult<ListMessages> {
    message::handlers::list_messages(list_messages_input)
}

// #[hdk_extern]
// fn what(_: ()) -> ChatResult<MessageInput> {
//     let eh = hash_entry!(ChannelEntry {
//         uuid: "".into(),
//         content: "".into()
//     })?;
//     let rt = Parent::Channel;
//     let me = MessageEntry {
//         uuid: "".into(),
//         content: "".into(),
//     };
//     let mi = MessageInput {
//         parent: rt,
//         channel_entry_hash: eh,
//         message: me,
//     };
//     Ok(mi)
// }
