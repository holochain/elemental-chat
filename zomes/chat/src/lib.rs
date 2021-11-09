pub use channel::{ChannelData, ChannelInfo, ChannelInput, ChannelList, ChannelListInput};
pub use entries::{channel, message};
pub use error::{ChatError, ChatResult};
pub use hc_joining_code;
pub use hdk::prelude::Path;
pub use hdk::prelude::*;
pub use message::{
    ActiveChatters, ListMessages, ListMessagesInput, Message, MessageData, MessageInput,
    SigResults, SignalMessageData, SignalSpecificInput,
};
pub mod entries;
pub mod error;
pub mod pagination_helper;
pub mod utils;
pub mod validation;

// signals:
pub const NEW_MESSAGE_SIGNAL_TYPE: &str = "new_message";
pub const NEW_CHANNEL_SIGNAL_TYPE: &str = "new_channel";

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
#[serde(tag = "signal_name", content = "signal_payload")]
pub enum SignalPayload {
    Message(SignalMessageData),
    Channel(ChannelData),
}

// pub(crate) fn _signal_ui(signal: SignalPayload) -> ChatResult<()> {
/*let signal_payload = match signal {
    SignalPayload::SignalMessageData(_) => SignalDetails {
        signal_name: "message".to_string(),
        signal_payload: signal,
    },
    SignalPayload::ChannelData(_) => SignalDetails {
        signal_name: "channel".to_string(),
        signal_payload: signal,
    },
};*/
// Ok(emit_signal(&signal)?)
// }

#[hdk_extern]
pub fn recv_remote_signal(signal: ExternIO) -> ExternResult<()> {
    let sig: SignalPayload = signal.decode()?;
    trace!("Received remote signal {:?}", sig);
    Ok(emit_signal(&sig)?)
}

entry_defs![
    Path::entry_def(),
    Message::entry_def(),
    ChannelInfo::entry_def()
];

#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // grant unrestricted access to accept_cap_claim so other agents can send us claims
    let mut functions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));
    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;
    validation::set_read_only_cap_tokens()?;
    if hc_joining_code::skip_proof() {
        Ok(InitCallbackResult::Pass)
    } else {
        return hc_joining_code::init_validate_and_create_joining_code();
    }
}

#[hdk_extern]
pub fn genesis_self_check(data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    if hc_joining_code::skip_proof_sb(&data.dna_def.properties) {
        return Ok(ValidateCallbackResult::Valid);
    }
    let holo_agent_key = hc_joining_code::holo_agent(&data.dna_def.properties)?;
    hc_joining_code::validate_joining_code(holo_agent_key, data.agent_key, data.membrane_proof)
}

#[hdk_extern]
pub fn create_channel(channel_input: ChannelInput) -> ExternResult<ChannelData> {
    if hc_joining_code::is_read_only_instance() {
        return Err(ChatError::ReadOnly.into());
    }
    Ok(channel::handlers::create_channel(channel_input)?)
}

#[hdk_extern]
pub fn validate(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    validation::common_validatation(data)
}

#[hdk_extern]
pub fn create_message(message_input: MessageInput) -> ExternResult<MessageData> {
    if hc_joining_code::is_read_only_instance() {
        return Err(ChatError::ReadOnly.into());
    }
    Ok(message::handlers::create_message(message_input)?)
}

/*#[hdk_extern]
fnpub  signal_users_on_channel(message_data SignalMessageData) -> ChatResult<()> {
    message::handlers::signal_users_on_channel(message_data)
}*/

#[hdk_extern]
pub fn get_active_chatters(_: ()) -> ExternResult<ActiveChatters> {
    Ok(message::handlers::get_active_chatters()?)
}

#[hdk_extern]
pub fn signal_specific_chatters(input: SignalSpecificInput) -> ExternResult<()> {
    if hc_joining_code::is_read_only_instance() {
        return Err(ChatError::ReadOnly.into());
    }
    Ok(message::handlers::signal_specific_chatters(input)?)
}

#[hdk_extern]
pub fn signal_chatters(message_data: SignalMessageData) -> ExternResult<SigResults> {
    if hc_joining_code::is_read_only_instance() {
        return Err(ChatError::ReadOnly.into());
    }
    Ok(message::handlers::signal_chatters(message_data)?)
}

#[hdk_extern]
pub fn refresh_chatter(_: ()) -> ExternResult<()> {
    Ok(message::handlers::refresh_chatter()?)
}

// #[hdk_extern]
// fn new_message_signal(message_input: SignalMessageData) -> ChatResult<()> {
//     message::handlers::new_message_signal(message_input)
// }

#[hdk_extern]
pub fn list_channels(list_channels_input: ChannelListInput) -> ExternResult<ChannelList> {
    Ok(channel::handlers::list_channels(list_channels_input)?)
}

#[hdk_extern]
pub fn list_messages(list_messages_input: ListMessagesInput) -> ExternResult<ListMessages> {
    Ok(message::handlers::list_messages(list_messages_input)?)
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessages {
    pub channel: ChannelData,
    pub messages: Vec<MessageData>,
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllMessagesList(pub Vec<ChannelMessages>);

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListAllMessagesInput {
    pub category: String,
    pub chunk: message::Chunk,
}

/// Deprecated
// #[hdk_extern]
// fn list_all_messages(input: ListAllMessagesInput) -> ExternResult<AllMessagesList> {
//     let channels = channel::handlers::list_channels(ChannelListInput {
//         category: input.category.clone(),
//     })?;
//     let all_messages: Result<Vec<ChannelMessages>, ChatError> = channels
//         .channels
//         .into_iter()
//         .map(|channel| {
//             let list_messages_input = ListMessagesInput {
//                 channel: channel.entry.clone(),
//                 chunk: input.chunk.clone(),
//                 active_chatter: false,
//             };
//             let messages = message::handlers::list_messages(list_messages_input)?;
//             Ok(ChannelMessages {
//                 channel,
//                 messages: messages.messages,
//             })
//         })
//         .collect();
//     Ok(AllMessagesList(all_messages?))
// }

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
pub struct Stats {
    agents: usize,
    active: usize,
    channels: usize,
    messages: usize,
}

#[hdk_extern]
pub fn stats(list_channels_input: ChannelListInput) -> ExternResult<Stats> {
    let (agents, active) = message::handlers::agent_stats()?;
    let (channels, messages) = channel::handlers::channel_stats(list_channels_input)?;
    Ok(Stats {
        agents,
        active,
        channels,
        messages,
    })
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
pub struct AgentStats {
    agents: usize,
    active: usize,
}
#[hdk_extern]
pub fn agent_stats(_: ()) -> ExternResult<AgentStats> {
    let (agents, active) = message::handlers::agent_stats()?;
    Ok(AgentStats { agents, active })
}
