pub use channel::{Channel, ChannelData, ChannelInfo, ChannelInput, ChannelList, ChannelListInput};
pub use entries::{channel, message};
pub use error::{ChatError, ChatResult};
pub use hc_joining_code;
pub use hdk::prelude::Path;
pub use hdk::prelude::*;
pub use message::{
    ActiveChatters, ListMessages, ListMessagesInput, Message, MessageData, MessageInput,
    SigResults, SignalMessageData, SignalSpecificInput,
};
pub mod batching_helper;
pub mod entries;
pub mod error;
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
fn recv_remote_signal(signal: ExternIO) -> ExternResult<()> {
    let sig: SignalPayload = signal.decode()?;
    trace!("Received remote signal {:?}", sig);
    Ok(emit_signal(&sig)?)
}

entry_defs![
    Path::entry_def(),
    PathEntry::entry_def(),
    Message::entry_def(),
    ChannelInfo::entry_def()
];

#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // grant unrestricted access to accept_cap_claim so other agents can send us claims
    let mut functions = BTreeSet::new();
    functions.insert((zome_info()?.name, "recv_remote_signal".into()));

    functions.insert((zome_info()?.name, "list_messages".into()));
    functions.insert((zome_info()?.name, "get_messages".into()));
    functions.insert((zome_info()?.name, "active_chatters".into()));
    functions.insert((zome_info()?.name, "get_active_chatters".into()));
    functions.insert((zome_info()?.name, "signal_specific_chatters".into()));
    functions.insert((zome_info()?.name, "signal_chatters".into()));
    functions.insert((zome_info()?.name, "is_active_chatter".into()));
    functions.insert((zome_info()?.name, "refresh_chatter".into()));
    functions.insert((zome_info()?.name, "agent_stats".into()));
    functions.insert((zome_info()?.name, "list_channels".into()));
    functions.insert((zome_info()?.name, "channel_stats".into()));

    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;
    Ok(InitCallbackResult::Pass)
}

#[hdk_extern]
fn create_channel(channel_input: ChannelInput) -> ExternResult<ChannelData> {
    if hc_joining_code::is_read_only_instance() {
        return Err(ChatError::ReadOnly.into());
    }
    Ok(channel::handlers::create_channel(channel_input)?)
}

#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    // validation::common_validatation(data)
    match op {
        Op::StoreEntry { entry, .. } => validation::__validate_create_entry(entry),
        _ => Ok(ValidateCallbackResult::Valid),
    }
}

#[hdk_extern]
fn insert_fake_messages(input: message::handlers::InsertFakeMessagesPayload) -> ExternResult<()> {
    message::handlers::insert_fake_messages(input)?;
    Ok(())
}

#[hdk_extern]
fn create_message(message_input: MessageInput) -> ExternResult<MessageData> {
    if hc_joining_code::is_read_only_instance() {
        return Err(ChatError::ReadOnly.into());
    }
    Ok(message::handlers::create_message(
        message_input,
        sys_time()?,
    )?)
}

/*#[hdk_extern]
fn signal_users_on_channel(message_data SignalMessageData) -> ChatResult<()> {
    message::handlers::signal_users_on_channel(message_data)
}*/

#[hdk_extern]
fn get_active_chatters(_: ()) -> ExternResult<ActiveChatters> {
    Ok(message::handlers::get_active_chatters()?)
}

#[hdk_extern]
fn signal_specific_chatters(input: SignalSpecificInput) -> ExternResult<()> {
    if hc_joining_code::is_read_only_instance() {
        return Err(ChatError::ReadOnly.into());
    }
    Ok(message::handlers::signal_specific_chatters(input)?)
}

#[hdk_extern]
fn signal_chatters(message_data: SignalMessageData) -> ExternResult<SigResults> {
    if hc_joining_code::is_read_only_instance() {
        return Err(ChatError::ReadOnly.into());
    }
    Ok(message::handlers::signal_chatters(message_data)?)
}

#[hdk_extern]
fn refresh_chatter(_: ()) -> ExternResult<()> {
    Ok(message::handlers::refresh_chatter()?)
}

// #[hdk_extern]
// fn new_message_signal(message_input: SignalMessageData) -> ChatResult<()> {
//     message::handlers::new_message_signal(message_input)
// }

#[hdk_extern]
fn list_channels(list_channels_input: ChannelListInput) -> ExternResult<ChannelList> {
    Ok(channel::handlers::list_channels(list_channels_input)?)
}

#[hdk_extern]
fn list_messages(list_messages_input: ListMessagesInput) -> ExternResult<ListMessages> {
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
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
pub struct Stats {
    agents: usize,
    active: usize,
    channels: usize,
    messages: usize,
}

#[hdk_extern]
fn stats(list_channels_input: ChannelListInput) -> ExternResult<Stats> {
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
fn agent_stats(_: ()) -> ExternResult<AgentStats> {
    let (agents, active) = message::handlers::agent_stats()?;
    Ok(AgentStats { agents, active })
}
