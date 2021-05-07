pub use channel::{ChannelData, ChannelInfo, ChannelInput, ChannelList, ChannelListInput};
pub use entries::{channel, message};
pub use error::{ChatResult, ChatError};
pub use hdk::prelude::Path;
pub use hdk::prelude::*;
pub use message::{
    ListMessages, ListMessagesInput, Message, MessageData, MessageInput, SigResults,
    SignalMessageData, SignalSpecificInput, ActiveChatters
};
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
    Message::entry_def(),
    ChannelInfo::entry_def()
];

#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    // grant unrestricted access to accept_cap_claim so other agents can send us claims
    let mut functions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));
    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;

    let entries = &query(ChainQueryFilter::new().header_type(HeaderType::AgentValidationPkg))?;
    if let Header::AgentValidationPkg(h) = entries[0].header() {
        match &h.membrane_proof {
            Some(mem_proof) => {
                let mem_proof = match Element::try_from(mem_proof.clone()) {
                    Ok(m) => m,
                    Err(_e) => return  Err(ChatError::InitFailure.into())
                };
                let code = mem_proof.entry().to_app_option::<validation::JoiningCodePayload>()?.unwrap();
                trace!("looking for {:?}", code.record_locator);
                let path = Path::from(code.record_locator.clone());
                if path.exists()? {
                    return Ok(InitCallbackResult::Fail(format!("membrane proof for {} already used", code.record_locator)))
                }
                trace!("creating {:?}", code.record_locator);
                path.ensure()?;
            },
            None => return Err(ChatError::InitFailure.into()),
        }
    } else {
        return Err(ChatError::InitFailure.into());
    }

    Ok(InitCallbackResult::Pass)
}
#[hdk_extern]
fn genesis_self_check(data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    validation::joining_code(data.agent_key, data.membrane_proof, true)
}
#[hdk_extern]
fn create_channel(channel_input: ChannelInput) -> ExternResult<ChannelData> {
    Ok(channel::handlers::create_channel(channel_input)?)
}

#[hdk_extern]
fn validate(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    validation::common_validatation(data)
}

#[hdk_extern]
fn create_message(message_input: MessageInput) -> ExternResult<MessageData> {
    Ok(message::handlers::create_message(message_input)?)
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
    Ok(message::handlers::signal_specific_chatters(input)?)
}

#[hdk_extern]
fn signal_chatters(message_data: SignalMessageData) -> ExternResult<SigResults> {
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
