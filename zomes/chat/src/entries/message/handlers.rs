use std::collections::{BTreeMap, VecDeque};

use crate::{
    error::ChatError,
    error::ChatResult,
    message::{Message, MessageInput},
    utils::get_local_header,
};
use hdk3::prelude::*;
use link::Link;
use metadata::EntryDetails;

use super::{ListMessages, ListMessagesInput, MessageEntry, ReplyKey, ReplyTo};

/// Create a new message
pub(crate) fn create_message(message_input: MessageInput) -> ChatResult<Message> {
    let MessageInput {
        reply_to,
        channel_entry_hash,
        message,
    } = message_input;

    // Commit the message
    let header_hash = create_entry!(&message)?;

    // Get the local header and create the message type for the UI
    let header = get_local_header(&header_hash)?.ok_or(ChatError::MissingLocalHeader)?;
    let hash_entry = header
        .entry_hash()
        .ok_or(ChatError::WrongHeaderType)?
        .clone();
    let message = Message::new(header, message, hash_entry.clone());

    // Get the hash of what this message is replying to
    let reply_to_hash_entry = match reply_to {
        ReplyTo::Message(hash_entry) => hash_entry,
        ReplyTo::Channel => channel_entry_hash.clone(),
    };

    // Turn the reply to and timestamp into a link tag
    let tag = ReplyKey::new(reply_to_hash_entry, message.created_at);
    create_link!(channel_entry_hash, hash_entry, LinkTag::from(tag))?;
    Ok(message)
}

/// List all the messages on this channel
pub(crate) fn list_messages(list_message_input: ListMessagesInput) -> ChatResult<ListMessages> {
    let channel_entry_hash = list_message_input.channel_entry_hash;
    // Get the message links on this channel
    let links = get_links!(channel_entry_hash.clone())?.into_inner();
    let len = links.len();

    // Our goal here is to sort the messages by who they replied to
    // and messages that replied to the same parent are ordered by time.

    // We can use the link tag to see who the message replied to and
    // the target will be the hash that child messages will have replied too.

    // This approach allows us to do one get_links call instead of a get_links
    // for each message.

    // Store links as HashMap reply to EntryHash -> Link
    let hash_to_link: BTreeMap<_, _> = links
        .into_iter()
        .map(|link| (ReplyKey::from(link.tag.clone()), link))
        .collect();

    // Create a sorted vec by following the "reply hash" -> Link -> target
    // starting from the channel entry hash
    let mut sorted_messages = Vec::with_capacity(len);
    let key: ReplyKey = channel_entry_hash.into();
    let mut keys = VecDeque::new();
    keys.push_back(key);
    while let Some(key) = keys.pop_front() {
        // Get all the messages at this "reply to" and sort them by time
        let sorted_by_time: BTreeMap<_, _> = hash_to_link
            .range(key.clone()..)
            .take_while(|(k, _)| k.reply_to_hash == key.reply_to_hash)
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
fn get_messages(links: Vec<Link>) -> ChatResult<Vec<Message>> {
    let mut messages = Vec::with_capacity(links.len());

    // for every link get details on the target and create the message
    for target in links.into_iter().map(|link| link.target) {
        // Get details because we are going to return the original message and
        // allow the UI to follow the CRUD tree to find which message
        // to actually display.
        let message = match get_details!(target)? {
            Some(Details::Entry(EntryDetails { entry, headers, .. })) => {
                // Turn the entry into a MessageEntry
                let message_entry: MessageEntry = entry.try_into()?;
                let header = match headers.into_iter().next() {
                    Some(h) => h,
                    // Ignoring missing messages
                    None => continue,
                };
                // Get the entry hash
                let hash_entry = header
                    .entry_hash()
                    .ok_or_else(|| {
                        ChatError::DataFormatError(
                            "The message is missing an entry type on the header",
                        )
                    })?
                    .clone();

                // Create the message type for the UI
                Message::new(header, message_entry, hash_entry)
            }
            // Message is missing. This could be an error but we are
            // going to ignore it.
            _ => continue,
        };
        messages.push(message);
    }
    Ok(messages)
}
