use std::collections::{BTreeMap, VecDeque};

use crate::{
    error::ChatError,
    error::ChatResult,
    message::{Message, MessageInput},
    signal_ui,
    utils::get_local_header,
    SignalPayload,
};
use hdk3::prelude::*;
use link::Link;
use metadata::EntryDetails;

use super::{
    LastSeen, LastSeenKey, ListMessages, ListMessagesInput, MessageData, SignalMessageData,
};

/// Create a new message
pub(crate) fn create_message(message_input: MessageInput) -> ChatResult<MessageData> {
    let MessageInput {
        last_seen,
        channel,
        message,
        chunk,
    } = message_input;

    // Commit the message
    let header_hash = create_entry(&message)?;

    // Get the local header and create the message type for the UI
    let header = get_local_header(&header_hash)?.ok_or(ChatError::MissingLocalHeader)?;
    let message = MessageData::new(header, message)?;

    // Get the channel hash
    let path: Path = channel.clone().into();

    // Add the current time components
    let path = add_chunk_path(path, chunk)?;

    // Ensure the path exists
    path.ensure()?;

    // The actual hash we are going to hang this message on
    let channel_entry_hash = path.hash()?;

    // Get the hash of the last_seen of this message
    let parent_hash_entry = match last_seen {
        LastSeen::Message(hash_entry) => hash_entry,
        LastSeen::First => channel_entry_hash.clone(),
    };

    // Turn the reply to and timestamp into a link tag
    let tag = LastSeenKey::new(parent_hash_entry, message.created_at);
    create_link(
        channel_entry_hash,
        message.entry_hash.clone(),
        LinkTag::from(tag),
    )?;

    // emit signal alerting all connected uis about new message
    signal_ui(SignalPayload::SignalMessageData(SignalMessageData::new(
        message.clone(),
        channel,
    )))?;

    // Return the message for the UI
    Ok(message)
}

/// List all the messages on this channel
pub(crate) fn list_messages(list_message_input: ListMessagesInput) -> ChatResult<ListMessages> {
    let ListMessagesInput { channel, chunk } = list_message_input;

    // Get the channel hash
    let path: Path = channel.into();

    // Add the chunk component
    let path = add_chunk_path(path, chunk)?;

    // Ensure the path exists
    path.ensure()?;

    // Get the actual hash we are going to pull the messages from
    let channel_entry_hash = path.hash()?;

    // Get the message links on this channel
    let links = get_links(channel_entry_hash.clone(), None)?.into_inner();
    let len = links.len();

    // Our goal here is to sort the messages by who they replied to
    // and messages that replied to the same last_seen are ordered by time.

    // We can use the link tag to see who the message replied to and
    // the target will be the hash that child messages will have replied too.

    // This approach allows us to do one get_links call instead of a get_links
    // for each message.

    // Store links as HashMap reply to EntryHash -> Link
    let hash_to_link: BTreeMap<_, _> = links
        .into_iter()
        .map(|link| (LastSeenKey::from(link.tag.clone()), link))
        .collect();

    // Create a sorted vec by following the "reply hash" -> Link -> target
    // starting from the channel entry hash
    let mut sorted_messages = Vec::with_capacity(len);
    let key: LastSeenKey = channel_entry_hash.into();
    let mut keys = VecDeque::new();
    keys.push_back(key);
    while let Some(key) = keys.pop_front() {
        // Get all the messages at this "reply to" and sort them by time
        let sorted_by_time: BTreeMap<_, _> = hash_to_link
            .range(key.clone()..)
            .take_while(|(k, _)| k.parent_hash == key.parent_hash)
            .map(|(k, v)| (k.timestamp, v))
            .collect();
        // Extend our sorted messages by these replies sorted by time
        // so we get messages sorted by the hash they replied to then sorted by time
        sorted_messages.extend(sorted_by_time.iter().map(|(_, v)| (*v).clone()));
        // Now we need to update the key to the next keys
        keys.extend(
            sorted_by_time
                .into_iter()
                .map(|(_, v)| v.target.clone().into()),
        );
    }
    let sorted_messages = get_messages(sorted_messages)?;
    Ok(sorted_messages.into())
}

// Turn all the link targets into the actual message
fn get_messages(links: Vec<Link>) -> ChatResult<Vec<MessageData>> {
    let mut messages = Vec::with_capacity(links.len());

    // for every link get details on the target and create the message
    for target in links.into_iter().map(|link| link.target) {
        // Get details because we are going to return the original message and
        // allow the UI to follow the CRUD tree to find which message
        // to actually display.
        let message = match get_details(target, GetOptions)? {
            Some(Details::Entry(EntryDetails {
                entry, mut headers, ..
            })) => {
                // Turn the entry into a MessageEntry
                let message: Message = entry.try_into()?;
                let signed_header = match headers.pop() {
                    Some(h) => h,
                    // Ignoring missing messages
                    None => continue,
                };

                // Create the message type for the UI
                MessageData::new(signed_header.header().clone(), message)?
            }
            // Message is missing. This could be an error but we are
            // going to ignore it.
            _ => continue,
        };
        messages.push(message);
    }
    Ok(messages)
}

/// Add the chunk index from the Date type to this path
fn add_chunk_path(path: Path, chunk: u32) -> ChatResult<Path> {
    let mut components: Vec<_> = path.into();

    components.push(format!("{}",chunk).into());
    Ok(components.into())
}
