use channel::{ChannelData, ChannelInfo, ChannelInput, ChannelList, ChannelListInput};
use entries::{channel, message};
use error::ChatResult;
use hdk3::prelude::Path;
use hdk3::prelude::*;
use message::{ListMessages, ListMessagesInput, Message, SignalMessageData, MessageData, MessageInput};

mod entries;
mod error;
mod utils;

// signals:
pub const NEW_MESSAGE_SIGNAL_TYPE: &str = "new_message";
pub const NEW_CHANNEL_SIGNAL_TYPE: &str = "new_channel";

#[derive(Serialize, Deserialize, SerializedBytes)]
enum SignalPayload {
	SignalMessageData(SignalMessageData),
	ChannelData(ChannelData),
}

#[derive(Serialize, Deserialize, SerializedBytes)]
struct SignalDetails {
    pub signal_name: String,
    pub signal_payload: SignalPayload,
}

 pub (crate) fn signal_ui(signal: SignalPayload) -> ChatResult<()> {
    let signal_payload = match signal {
        SignalPayload::SignalMessageData(_) => SignalDetails {
				signal_name: "message".to_string(),
				signal_payload: signal
		},
        SignalPayload::ChannelData(_) => SignalDetails {
				signal_name: "channel".to_string(),
				signal_payload: signal
		}
    };
    let payload: SerializedBytes = SerializedBytes::try_from(signal_payload).unwrap();
	Ok(emit_signal!(payload)?)
}

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
fn list_channels(list_channels_input: ChannelListInput) -> ChatResult<ChannelList> {
    channel::handlers::list_channels(list_channels_input)
}

#[hdk_extern]
fn list_messages(list_messages_input: ListMessagesInput) -> ChatResult<ListMessages> {
    message::handlers::list_messages(list_messages_input)
}
