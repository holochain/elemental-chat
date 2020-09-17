use crate::{
    channel::{Channel, ChannelInput},
    error::ChatResult,
    utils::to_timestamp,
};
use hdk3::prelude::*;
use link::Link;

use super::{ChannelData, ChannelInfo, ChannelInfoTag, ListChannels, ListChannelsInput};

/// Create a new channel
pub(crate) fn create_channel(channel_input: ChannelInput) -> ChatResult<ChannelData> {
    let ChannelInput { name, channel } = channel_input;

    // Create the path for this channel
    let path: Path = channel.clone().into();

    path.ensure()?;

    // Commit the channel info
    let created_at = to_timestamp(sys_time!()?);
    let info = ChannelInfo {
        created_by: agent_info!()?.agent_initial_pubkey,
        created_at,
        name,
    };

    create_entry!(&info)?;
    let info_hash = hash_entry!(&info)?;

    // link the channel to the path
    create_link!(path.hash()?, info_hash, ChannelInfoTag::tag())?;
    Ok(ChannelData::new(channel, info))
}

pub(crate) fn list_channels(list_channels_input: ListChannelsInput) -> ChatResult<ListChannels> {
    // Make sure the path exists
    let path = Path::from(list_channels_input.category.clone());

    // Get any channels on this path
    let links = path.children()?.into_inner();
    let mut channels = Vec::with_capacity(links.len());
    for tag in links.into_iter().map(|link| link.tag) {
        let channel_path = Path::try_from(&tag)?;
        let channel = Channel::try_from(&channel_path)?;
        let channel_info = get_links!(channel_path.hash()?, ChannelInfoTag::tag())?.into_inner();
        let latest_info = channel_info
            .into_iter()
            .fold(None, |latest: Option<Link>, link| match latest {
                Some(latest) => {
                    if link.timestamp > latest.timestamp {
                        Some(link)
                    } else {
                        Some(latest)
                    }
                }
                None => Some(link),
            });
        let latest_info = match latest_info {
            Some(l) => l,
            None => continue,
        };
        if let Some(element) = get!(latest_info.target)? {
            if let Some(info) = element.into_inner().1.to_app_option()? {
                channels.push(ChannelData { channel, info });
            }
        }
    }
    Ok(channels.into())
}
// pub(crate) fn list_channels(list_channels_input: ListChannelsInput) -> ChatResult<ListChannels> {
//     // Make sure the path exists
//     let path = Path::from(list_channels_input.category.clone());
//     // path.ensure()?;

//     // Get any channels on this path
//     let links = path.children()?.into_inner();
//     // let links = get_links!(path.hash()?)?.into_inner();

//     // Collect all the channels that haven't been deleted
//     let mut channels = Vec::with_capacity(links.len());
//     for target in links.into_iter().map(|link| link.target) {
//         // Get details on this channel because we want collapse the CRUD possibilities.
//         // Rules are:
//         // - If there are any deletes then the channel is deleted
//         // - If there are updates then follow them always choosing
//         //   the one with the latest timestamp
//         let channel = match get_details!(target)? {
//             Some(Details::Entry(EntryDetails {
//                 deletes,
//                 updates,
//                 entry,
//                 headers,
//                 ..
//             })) => {
//                 // Channel is deleted so skip it
//                 if !deletes.is_empty() {
//                     continue;
//                 }
//                 // No updates so return this channel
//                 if updates.is_empty() {
//                     let channel_entry: ChannelEntry = entry.try_into()?;
//                     let header = headers.into_iter().next().ok_or_else(|| {
//                         ChatError::MissingChannel(list_channels_input.path.clone())
//                     })?;
//                     let hash_entry = header
//                         .entry_hash()
//                         .ok_or_else(|| {
//                             ChatError::DataFormatError(
//                                 "The channel is missing an entry type on the header",
//                             )
//                         })?
//                         .clone();
//                     Channel::new(header, channel_entry, hash_entry)
//                 } else {
//                     // Updates are a future feature (or homework ;) )
//                     todo!("follow update chain choosing latest update")
//                 }
//             }
//             // Channel is just missing. This could be an error
//             // but we are going to ignore it.
//             _ => continue,
//         };
//         channels.push(channel);
//     }
//     Ok(ListChannels { channels })
// }
