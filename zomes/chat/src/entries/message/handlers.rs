use std::collections::{BTreeMap, VecDeque};

use crate::{
    error::ChatError,
    error::ChatResult,
    message::{Message, MessageInput},
    utils::get_local_header,
    utils::to_date,
};
use hdk3::prelude::*;
use link::Link;
use metadata::EntryDetails;

use super::{Date, ListMessages, ListMessagesInput, MessageData, Parent, ParentKey};

/// Create a new message
pub(crate) fn create_message(message_input: MessageInput) -> ChatResult<MessageData> {
    let MessageInput {
        parent,
        channel,
        message,
    } = message_input;

    // Commit the message
    let header_hash = create_entry!(&message)?;

    // Get the local header and create the message type for the UI
    let header = get_local_header(&header_hash)?.ok_or(ChatError::MissingLocalHeader)?;
    let message = MessageData::new(header, message)?;

    // Get the channel hash
    let path: Path = channel.clone().into();
    let path = add_current_time_path(path)?;
    path.ensure()?;

    let channel_entry_hash = path.hash()?;

    // Get the hash of what this message is replying to
    let parent_hash_entry = match parent {
        Parent::Message(hash_entry) => hash_entry,
        Parent::First => channel_entry_hash.clone(),
    };

    // Turn the reply to and timestamp into a link tag
    let tag = ParentKey::new(parent_hash_entry, message.created_at);
    create_link!(
        channel_entry_hash,
        message.entry_hash.clone(),
        LinkTag::from(tag)
    )?;
    Ok(message)
}

/// List all the messages on this channel
pub(crate) fn list_messages(list_message_input: ListMessagesInput) -> ChatResult<ListMessages> {
    let ListMessagesInput { channel, date } = list_message_input;

    // Get the channel hash
    let path: Path = channel.clone().into();
    let path = add_time_path(path, date)?;
    path.ensure()?;

    let channel_entry_hash = path.hash()?;

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
        .map(|link| (ParentKey::from(link.tag.clone()), link))
        .collect();

    // Create a sorted vec by following the "reply hash" -> Link -> target
    // starting from the channel entry hash
    let mut sorted_messages = Vec::with_capacity(len);
    let key: ParentKey = channel_entry_hash.into();
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
        let message = match get_details!(target)? {
            Some(Details::Entry(EntryDetails { entry, headers, .. })) => {
                // Turn the entry into a MessageEntry
                let message: Message = entry.try_into()?;
                let header = match headers.into_iter().next() {
                    Some(h) => h,
                    // Ignoring missing messages
                    None => continue,
                };

                // Create the message type for the UI
                MessageData::new(header, message)?
            }
            // Message is missing. This could be an error but we are
            // going to ignore it.
            _ => continue,
        };
        messages.push(message);
    }
    Ok(messages)
}

fn add_time_path(path: Path, date: Date) -> ChatResult<Path> {
    let Date { year, month, day } = date;
    let mut components: Vec<_> = path.into();
    debug!(format!("{}{}{}", year, month, day))?;
    components.push(year.to_string().into());
    components.push(month.to_string().into());
    components.push(day.to_string().into());
    Ok(components.into())
}

fn add_current_time_path(path: Path) -> ChatResult<Path> {
    use chrono::Datelike;
    let mut components: Vec<_> = path.into();
    let year = to_date(sys_time!()?).year().to_string();
    let month = to_date(sys_time!()?).month().to_string();
    let day = to_date(sys_time!()?).day().to_string();
    debug!(format!("{}{}{}", year, month, day))?;
    components.push(year.into());
    components.push(month.into());
    components.push(day.into());
    Ok(components.into())
}
