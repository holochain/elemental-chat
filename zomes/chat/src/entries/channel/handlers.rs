use crate::{
    channel::{Channel, ChannelInput},
    error::ChatError,
    error::ChatResult,
    utils::get_local_header,
};
use hdk3::prelude::*;
use metadata::EntryDetails;

use super::{ChannelEntry, ListChannels, ListChannelsInput};

/// Create a new channel
pub(crate) fn create_channel(channel_input: ChannelInput) -> ChatResult<Channel> {
    // Create the path for this channel
    let path = Path::from(channel_input.path);
    path.ensure()?;

    // Commit the channel entry
    let header_hash = create_entry!(&channel_input.channel)?;

    // Get the header from the chain
    let header = get_local_header(&header_hash)?.ok_or(ChatError::MissingLocalHeader)?;
    let hash_entry = header
        .entry_hash()
        .ok_or(ChatError::WrongHeaderType)?
        .clone();

    // link the channel to the path
    create_link!(path.hash()?, hash_entry.clone())?;

    // Create the channel to return to the UI
    let channel = Channel::new(header, channel_input.channel, hash_entry);
    Ok(channel)
}

pub(crate) fn list_channels(list_channels_input: ListChannelsInput) -> ChatResult<ListChannels> {
    // Make sure the path exists
    let path = Path::from(list_channels_input.path.clone());
    path.ensure()?;

    // Get any channels on this path
    let links = get_links!(path.hash()?)?.into_inner();

    // Collect all the channels that haven't been deleted
    let mut channels = Vec::with_capacity(links.len());
    for target in links.into_iter().map(|link| link.target) {
        // Get details on this channel because we want collapse the CRUD possibilities.
        // Rules are:
        // - If there are any deletes then the channel is deleted
        // - If there are updates then follow them always choosing
        //   the one with the latest timestamp
        let channel = match get_details!(target)? {
            Some(Details::Entry(EntryDetails {
                deletes,
                updates,
                entry,
                headers,
                ..
            })) => {
                // Channel is deleted so skip it
                if !deletes.is_empty() {
                    continue;
                }
                // No updates so return this channel
                if updates.is_empty() {
                    let channel_entry: ChannelEntry = entry.try_into()?;
                    let header = headers.into_iter().next().ok_or_else(|| {
                        ChatError::MissingChannel(list_channels_input.path.clone())
                    })?;
                    let hash_entry = header
                        .entry_hash()
                        .ok_or_else(|| {
                            ChatError::DataFormatError(
                                "The channel is missing an entry type on the header",
                            )
                        })?
                        .clone();
                    Channel::new(header, channel_entry, hash_entry)
                } else {
                    // Updates are a future feature (or homework ;) )
                    todo!("follow update chain choosing latest update")
                }
            }
            // Channel is just missing. This could be an error
            // but we are going to ignore it.
            _ => continue,
        };
        channels.push(channel);
    }
    Ok(ListChannels { channels })
}
