use crate::{
    channel::{Channel, ChannelInput},
    error::ChatError,
    error::ChatResult,
    utils::get_local_header,
};
use hdk3::prelude::*;
use metadata::EntryDetails;

use super::{ChannelEntry, ListChannels, ListChannelsInput};

pub(crate) fn create_channel(channel_input: ChannelInput) -> ChatResult<Channel> {
    let path = Path::from(channel_input.path);
    path.ensure()?;
    let header_hash = create_entry!(&channel_input.channel)?;
    let header = get_local_header(&header_hash)?
        .ok_or(ChatError::MissingLocalHeader)?
        .into_content();
    let hash_entry = header
        .entry_hash()
        .ok_or(ChatError::WrongHeaderType)?
        .clone();
    let channel = Channel::new(header, channel_input.channel, hash_entry.clone());
    create_link!(path.hash()?, hash_entry)?;
    Ok(channel)
}

pub(crate) fn list_channels(list_channels_input: ListChannelsInput) -> ChatResult<ListChannels> {
    let path = Path::from(list_channels_input.path);
    path.ensure()?;
    let links = get_links!(path.hash()?)?.into_inner();
    let mut channels = vec![];
    for target in links.into_iter().map(|link| link.target) {
        let channel = match get_details!(target)? {
            Some(Details::Entry(EntryDetails {
                deletes,
                updates,
                entry,
                headers,
                ..
            })) => {
                if !deletes.is_empty() {
                    continue;
                }
                if updates.is_empty() {
                    let channel_entry: ChannelEntry = entry.try_into()?;
                    let header = headers
                        .into_iter()
                        .next()
                        .expect("Why is there no headers?");
                    let hash_entry = header
                        .entry_hash()
                        .expect("why is there no entry hash?")
                        .clone();
                    Channel::new(header, channel_entry, hash_entry)
                } else {
                    loop {
                        // updates.sort_by_key(|eu| eu.timestamp);
                        // updates
                        //     .first()
                        //     .expect("you said you weren't empty")
                        //     .hash_entry;
                        break;
                    }
                    todo!("follow update chain choosing latest update")
                }
            }
            _ => continue,
        };
        channels.push(channel);
    }
    Ok(ListChannels { channels })
}
