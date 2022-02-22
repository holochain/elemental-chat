use crate::{
    error::ChatError,
    error::ChatResult,
    message::{Message, MessageInput},
    utils::{get_local_header, to_date},
    SignalPayload,
};
use hdk::prelude::*;
use link::Link;
use metadata::EntryDetails;

use super::{
    ActiveChatters, LastSeen, LastSeenKey, ListMessages, ListMessagesInput, MessageData,
    SigResults, SignalMessageData, SignalSpecificInput,
};

/// Create a new message
pub(crate) fn create_message(message_input: MessageInput) -> ChatResult<MessageData> {
    let MessageInput {
        last_seen,
        channel,
        entry,
        ..
    } = message_input;

    // Commit the message
    let header_hash = create_entry(&entry)?;

    // Get the local header and create the message type for the UI
    let header = get_local_header(&header_hash)?.ok_or(ChatError::MissingLocalHeader)?;
    let message = MessageData::new(header, entry)?;

    // Get the channel hash
    let path: Path = channel.clone().into();

    // Add the current time components
    let path = crate::batching_helper::timestamp_into_path(path, sys_time()?)?;

    // Ensure the path exists
    path.ensure()?;

    // The actual hash we are going to hang this message on
    let path_hash = path.path_entry_hash()?;
    debug!(
        "committing message to hour {:?}",
        crate::batching_helper::last_segment_from_path(&path)?
    );
    // Get the hash of the last_seen of this message
    let parent_hash_entry = match last_seen {
        LastSeen::Message(hash_entry) => hash_entry,
        LastSeen::First => path_hash.clone(),
    };
    // Turn the reply to and timestamp into a link tag
    let tag = LastSeenKey::new(parent_hash_entry, message.created_at);
    create_link(path_hash, message.entry_hash.clone(), LinkTag::from(tag))?;

    // Return the message for the UI
    Ok(message)
}

/// Using batching to List all the messages on this channel
pub(crate) fn list_messages(list_message_input: ListMessagesInput) -> ChatResult<ListMessages> {
    let ListMessagesInput {
        channel,
        earliest_seen,
        target_message_count,
    } = list_message_input;

    let path: Path = channel.into();
    let mut links =
        crate::batching_helper::get_message_links(path, earliest_seen, target_message_count)?;
    links.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let sorted_messages = get_messages(links)?;
    Ok(sorted_messages.into())
}

// pub(crate) fn _new_message_signal(message: SignalMessageData) -> ChatResult<()> {
//     debug!(
//         "Received message: {:?}",
//         message.message_data.message.content
//     );
// emit signal alerting all connected uis about new message
// signal_ui(SignalPayload::Message(message))
// }

// Turn all the link targets into the actual message
fn get_messages(links: Vec<Link>) -> ChatResult<Vec<MessageData>> {
    // Optimizing by calling parallel gets
    let mut messages = Vec::with_capacity(links.len());
    // for every link get details on the target and create the message
    let msg_results_input: Vec<GetInput> = links
        .into_iter()
        .map(|link| GetInput::new(link.target.into(), GetOptions::default()))
        .collect();
    let all_msg_results_elements = HDK.with(|hdk| hdk.borrow().get_details(msg_results_input))?;

    for ele in all_msg_results_elements.into_iter() {
        match ele {
            Some(Details::Entry(EntryDetails {
                entry, mut headers, ..
            })) => {
                // Turn the entry into a MessageEntry
                let message: Message = entry.try_into()?;
                let signed_header = match headers.pop() {
                    Some(h) => h,
                    // Ignoring missing messages
                    None => {
                        debug!("Ignoring missing messages");
                        continue;
                    }
                };

                // Create the message type for the UI
                messages.push(MessageData::new(signed_header.header().clone(), message)?)
            }
            // Message is missing. This could be an error but we are
            // going to ignore it.
            _ => continue, // Create the message type for the UI
        }
    }
    Ok(messages)
}

pub fn chatters_path() -> Path {
    Path::from("chatters")
}

/*  At some point maybe add back in chatters  on a channel, but for now
simple list of global chatters.
pub(crate) fn signal_users_on_channel(signal_message_data: SignalMessageData) -> ChatResult<()> {
    let me = agent_info()?.agent_latest_pubkey;

    let path: Path = signal_message_data.channel_data.channel.chatters_path();
    let hour_path = add_current_hour_path(path.clone())?;
    hour_path.ensure()?;
    signal_hour(hour_path, signal_message_data.clone(), me.clone())?;
    let hour_path = add_current_hour_minus_n_path(path, 1)?;
    hour_path.ensure()?;
    signal_hour(hour_path, signal_message_data, me)?;

    let path: Path = chatters_path();
    signal_chatters(path, signal_message_data, me)?;

    Ok(())
} */

const CHATTER_REFRESH_HOURS: i64 = 2;

use std::collections::HashSet;

/// return the list of active chatters on a path.
/// N.B.: assumes that the path has been ensured elsewhere.
fn active_chatters(chatters_path: Path) -> ChatResult<(usize, Vec<AgentPubKey>)> {
    let chatters = get_links(chatters_path.path_entry_hash()?, None)?;
    debug!("num online chatters {}", chatters.len());
    let now = to_date(sys_time()?);
    let total = chatters.len();
    let mut agents = HashSet::new();
    let active: Vec<AgentPubKey> = chatters
        .into_iter()
        .map(|l| {
            let link_time: chrono::DateTime<chrono::Utc> = l.timestamp.try_into()?;
            let maybe_agent =
                if now.signed_duration_since(link_time).num_hours() < CHATTER_REFRESH_HOURS {
                    let tag = l.tag;
                    if let Ok(agent) = tag_to_agent(tag) {
                        if agents.contains(&agent) {
                            None
                        } else {
                            agents.insert(agent.clone());
                            Some(agent)
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
            ChatResult::Ok(maybe_agent)
        })
        .collect::<Result<Vec<Option<AgentPubKey>>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok((total, active))
}

pub(crate) fn get_active_chatters() -> ChatResult<ActiveChatters> {
    let me = agent_info()?.agent_latest_pubkey;
    let chatters_path: Path = chatters_path();
    let (_total, mut chatters) = active_chatters(chatters_path)?;
    chatters.retain(|a| *a != me);
    Ok(ActiveChatters { chatters })
}

pub(crate) fn signal_specific_chatters(input: SignalSpecificInput) -> ChatResult<()> {
    let mut chatters = input.chatters;

    if let Some(include_active_chatters) = input.include_active_chatters {
        if include_active_chatters {
            let active_chatters_result = get_active_chatters();
            if let Ok(mut active_chatters) = active_chatters_result {
                chatters.append(&mut active_chatters.chatters);
            }
        }
    }
    let input = SignalPayload::Message(input.signal_message_data);
    let payload = ExternIO::encode(input)?;
    remote_signal(payload, chatters)?;
    Ok(())
}

pub(crate) fn signal_chatters(signal_message_data: SignalMessageData) -> ChatResult<SigResults> {
    let me = agent_info()?.agent_latest_pubkey;
    let chatters_path: Path = chatters_path();
    let (total, mut active_chatters) = active_chatters(chatters_path)?;
    active_chatters.retain(|a| *a != me);
    debug!("sending to {:?}", active_chatters);

    let mut sent: Vec<String> = Vec::new();
    for a in active_chatters.clone() {
        sent.push(format!("{}", a.to_string()));
    }
    let input = SignalPayload::Message(signal_message_data);
    let payload = ExternIO::encode(input)?;
    remote_signal(payload, active_chatters)?;
    Ok(SigResults { total, sent })
}

pub(crate) fn is_active_chatter(chatters_path: Path) -> ChatResult<bool> {
    let base = chatters_path.path_entry_hash()?;
    let filter = QueryFilter::new();
    let header_filter = filter.header_type(HeaderType::CreateLink);
    let query_result: Vec<Element> = query(header_filter)?;
    let now = to_date(sys_time()?);
    let mut pass = false;
    for x in query_result {
        match x.header() {
            Header::CreateLink(c) => {
                if c.base_address == base {
                    let link_time = to_date(c.timestamp);
                    if now.signed_duration_since(link_time).num_hours() < CHATTER_REFRESH_HOURS {
                        pass = true;
                        break;
                    } else {
                        pass = false;
                        break;
                    }
                } else {
                    continue;
                }
            }
            _ => unreachable!(),
        }
    }
    Ok(pass)
}

// TODO: re add chatter/channel instead of global
// simplified and expected as a zome call
pub(crate) fn refresh_chatter() -> ChatResult<()> {
    let path: Path = chatters_path();
    path.ensure()?;
    let agent = agent_info()?.agent_latest_pubkey;
    let agent_tag = agent_to_tag(&agent);
    if !is_active_chatter(path.clone())? {
        create_link(path.path_entry_hash()?, agent.into(), agent_tag.clone())?;
    }
    Ok(())
}

// this is a relatively expensive call and really only for testing purposes
pub(crate) fn agent_stats() -> ChatResult<(usize, usize)> {
    let chatters_path: Path = chatters_path();
    let chatters = get_links(chatters_path.path_entry_hash()?, None)?;

    let agents = chatters
        .into_iter()
        .map(|l| l.tag)
        .collect::<::std::collections::HashSet<_>>();

    let (_, active_chatters) = active_chatters(chatters_path)?;
    Ok((agents.len(), active_chatters.len()))
}

/* old way using hours
fn add_chatter(path: Path) -> ChatResult<()> {
    let agent = agent_info()?.agent_latest_pubkey;
    let agent_tag = agent_to_tag(&agent);

    let hour_path = add_current_hour_path(path.clone())?;
    hour_path.ensure()?;
    let my_chatter = get_links(hour_path.hash()?, Some(agent_tag.clone()))?.into_inner();
    debug!("checking chatters");
    if my_chatter.is_empty() {
        debug!("adding chatters");
        create_link(hour_path.hash()?, agent.into(), agent_tag.clone())?;
        let hour_path = add_current_hour_minus_n_path(path, 1)?;
        hour_path.ensure()?;
        for link in get_links(hour_path.hash()?, Some(agent_tag.clone()))?.into_inner() {
            delete_link(link.create_link_hash)?;
        }
    }

    Ok(())
}
*/

fn agent_to_tag(agent: &AgentPubKey) -> LinkTag {
    let agent_tag: &[u8] = agent.as_ref();
    LinkTag::new(agent_tag)
}

fn tag_to_agent(tag: LinkTag) -> ChatResult<AgentPubKey> {
    Ok(AgentPubKey::from_raw_39(tag.0).map_err(|_| ChatError::AgentTag)?)
}

/*
fn add_current_hour_path(path: Path) -> ChatResult<Path> {
    add_current_hour_path_inner(path, None)
}

fn add_current_hour_minus_n_path(path: Path, sub: u64) -> ChatResult<Path> {
    add_current_hour_path_inner(path, Some(sub))
}

fn add_current_hour_path_inner(path: Path, sub: Option<u64>) -> ChatResult<Path> {
    use chrono::{Datelike, Timelike};
    let mut components: Vec<_> = path.into();

    // Get the current times and turn them to dates;
    let mut now = to_date(sys_time()?);
    if let Some(sub) = sub {
        now = date_minus_hours(now, sub);
    }
    let year = now.year().to_string();
    let month = now.month().to_string();
    let day = now.day().to_string();
    let hour = now.hour().to_string();

    // Add the date parts as components to the path
    components.push(year.into());
    components.push(month.into());
    components.push(day.into());
    components.push(hour.into());
    Ok(components.into())
}

fn date_minus_hours(
    date: chrono::DateTime<chrono::Utc>,
    hours: u64,
) -> chrono::DateTime<chrono::Utc> {
    let hours = chrono::Duration::hours(hours as i64);
    date - hours
}
*/
