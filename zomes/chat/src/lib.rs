use channel::{ChannelData, ChannelInfo, ChannelInput, ChannelList, ChannelListInput};
use entries::{channel, message};
use error::ChatResult;
use hdk3::prelude::Path;
use hdk3::prelude::*;
use message::{
    ListMessages, ListMessagesInput, Message, MessageData, MessageInput, SigResults,
    SignalMessageData, SignalSpecificInput, ActiveChatters
};

mod entries;
mod error;
mod utils;
mod validate;

// signals:
pub const NEW_MESSAGE_SIGNAL_TYPE: &str = "new_message";
pub const NEW_CHANNEL_SIGNAL_TYPE: &str = "new_channel";

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
#[serde(tag = "signal_name", content = "signal_payload")]
enum SignalPayload {
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
fn recv_remote_signal(signal: SerializedBytes) -> ChatResult<()> {
    debug!(format!("Received remote signal"));
    let sig: SignalPayload = SignalPayload::try_from(signal.clone())?;
    debug!(format!("Received remote signal {:?}", sig));
    Ok(emit_signal(&signal)?)
}

#[hdk_extern]
fn signing(_: ()) -> ChatResult<Signature> {
    let me = agent_info()?.agent_latest_pubkey;
    debug!("ME: {:?}", me);
    let signature = sign(me.clone(), SerializedBytes::try_from(me.clone())?)?;
    debug!("Signature: {:?}", signature);

    let v = verify_signature(me.clone(), signature.clone(), SerializedBytes::try_from(me)?)?;
    debug!("Verify: {:?}", v);


    return Ok(signature)
}


entry_defs![
    Path::entry_def(),
    Message::entry_def(),
    ChannelInfo::entry_def()
];

#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // validate joining code
    validate::joining_code().unwrap();

    // grant unrestricted access to accept_cap_claim so other agents can send us claims
    let mut functions: GrantedFunctions = HashSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));
    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;

    Ok(InitCallbackResult::Pass)
}

#[hdk_extern]
fn create_channel(channel_input: ChannelInput) -> ChatResult<ChannelData> {
    channel::handlers::create_channel(channel_input)
}

#[hdk_extern]
fn validate(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    validate::common_validatation(data)
}

#[hdk_extern]
fn create_message(message_input: MessageInput) -> ChatResult<MessageData> {
    message::handlers::create_message(message_input)
}

/*#[hdk_extern]
fn signal_users_on_channel(message_data SignalMessageData) -> ChatResult<()> {
    message::handlers::signal_users_on_channel(message_data)
}*/

#[hdk_extern]
fn get_active_chatters(_: ()) -> ChatResult<ActiveChatters> {
    message::handlers::get_active_chatters()
}

#[hdk_extern]
fn signal_specific_chatters(input: SignalSpecificInput) -> ChatResult<()> {
    message::handlers::signal_specific_chatters(input)
}

#[hdk_extern]
fn signal_chatters(message_data: SignalMessageData) -> ChatResult<SigResults> {
    message::handlers::signal_chatters(message_data)
}

#[hdk_extern]
fn refresh_chatter(_: ()) -> ChatResult<()> {
    message::handlers::refresh_chatter()
}

// #[hdk_extern]
// fn new_message_signal(message_input: SignalMessageData) -> ChatResult<()> {
//     message::handlers::new_message_signal(message_input)
// }

#[hdk_extern]
fn list_channels(list_channels_input: ChannelListInput) -> ChatResult<ChannelList> {
    channel::handlers::list_channels(list_channels_input)
}

#[hdk_extern]
fn list_messages(list_messages_input: ListMessagesInput) -> ChatResult<ListMessages> {
    message::handlers::list_messages(list_messages_input)
}

#[derive(Serialize, Deserialize, SerializedBytes, Debug)]
pub struct Stats {
    agents: usize,
    active: usize,
    channels: usize,
    messages: usize,
}

#[hdk_extern]
fn stats(list_channels_input: ChannelListInput) -> ChatResult<Stats> {
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
fn agent_stats(_: ()) -> ChatResult<AgentStats> {
    let (agents, active) = message::handlers::agent_stats()?;
    Ok(AgentStats { agents, active })
}
