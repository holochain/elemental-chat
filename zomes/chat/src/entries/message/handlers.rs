use crate::{
    error::ChatError,
    error::ChatResult,
    message::{Message, MessageInput},
    utils::get_local_header,
};
use hdk3::prelude::*;
use link::Link;
use metadata::EntryDetails;

use super::{ListMessages, ListMessagesInput, MessageEntry, ReplyTo};

pub(crate) fn create_message(message_input: MessageInput) -> ChatResult<Message> {
    let header_hash = commit_entry!(&message_input.message)?;
    let header = get_local_header(&header_hash)?
        .ok_or(ChatError::MissingLocalHeader)?
        .into_content();
    let entry_hash = header
        .entry_hash()
        .ok_or(ChatError::WrongHeaderType)?
        .clone();
    let message = Message::new(header, message_input.message, entry_hash.clone());
    let reply_to_entry_hash = match message_input.reply_to {
        ReplyTo::Channel(channel_entry_hash) => match get_details!(channel_entry_hash.clone())? {
            Some(Details::Entry(EntryDetails {
                deletes,
                mut updates,
                ..
            })) => {
                if !deletes.is_empty() {
                    return Err(ChatError::ChannelDeleted);
                }
                if updates.is_empty() {
                    channel_entry_hash.clone()
                } else {
                    updates.sort_by_key(|eu| eu.timestamp);
                    updates
                        .first()
                        .expect("you said you weren't empty")
                        .entry_hash
                        .clone()
                }
            }
            _ => panic!("Can't get the channel"),
        },
        ReplyTo::Message(entry_hash) => entry_hash,
    };
    link_entries!(reply_to_entry_hash, entry_hash)?;
    Ok(message)
}

pub(crate) fn list_messages(list_message_input: ListMessagesInput) -> ChatResult<ListMessages> {
    let channel_entry_hash = list_message_input.channel_entry_hash;
    let mut processed = vec![];
    let mut pending = vec![];
    let mut now = vec![];
    let links = get_links!(channel_entry_hash)?.into_inner();
    now.extend(get_messages(links)?);
    loop {
        for msg in now {
            let links = get_links!(msg.entry_hash.clone())?.into_inner();
            pending.extend(get_messages(links)?);
            processed.push(msg);
        }
        now = pending;
        pending = vec![];
        if now.is_empty() {
            break;
        }
    }
    Ok(ListMessages {
        messages: processed,
    })
}

fn get_messages(links: Vec<Link>) -> ChatResult<Vec<Message>> {
    let mut messages = vec![];
    for target in links.into_iter().map(|link| link.target) {
        let message = match get_details!(target)? {
            Some(Details::Entry(EntryDetails {
                updates,
                entry,
                headers,
                ..
            })) => {
                if updates.is_empty() {
                    let message_entry: MessageEntry = entry.try_into()?;
                    let header = headers
                        .into_iter()
                        .next()
                        .expect("Why is there no headers?");
                    let entry_hash = header
                        .entry_hash()
                        .expect("why is there no entry hash?")
                        .clone();
                    Message::new(header, message_entry, entry_hash)
                } else {
                    todo!("Return all updates but wrapped in an Update enum")
                }
            }
            _ => continue,
        };
        messages.push(message);
    }
    Ok(messages)
}
