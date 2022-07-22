use super::{ChannelData, ChannelInfo, ChannelInfoTag, ChannelList, ChannelListInput};
use crate::{
    channel::{Channel, ChannelInput},
    error::ChatResult,
};
use hdk::hash_path::path::Component;
use hdk::prelude::*;
use link::Link;

/// Create a new channel
/// This effectively just stores channel info on the
/// path that is `category:channel_id`
pub(crate) fn create_channel(channel_input: ChannelInput) -> ChatResult<ChannelData> {
    let ChannelInput { name, entry } = channel_input;

    // Create the path for this channel
    let path: Path = entry.clone().into();
    path.ensure()?;

    // Create the channel info
    let info = ChannelInfo {
        category: entry.category.clone(),
        uuid: entry.uuid.clone(),
        // This agent
        created_by: agent_info()?.agent_initial_pubkey,
        // Right now
        created_at: sys_time()?,
        name,
    };

    // Commit the channel info
    create_entry(&info)?;
    let info_hash = hash_entry(&info)?;

    // link the channel info to the path
    create_link(
        path.path_entry_hash()?,
        info_hash,
        HdkLinkType::Any,
        ChannelInfoTag::tag(),
    )?;

    // Return the channel and the info for the UI
    Ok(ChannelData::new(entry, info))
}

fn category_path(category: String) -> Path {
    let path = vec![Component::from(category.as_bytes().to_vec())];
    Path::from(path)
}

pub(crate) fn list_channels(list_channels_input: ChannelListInput) -> ChatResult<ChannelList> {
    // Get the category path
    let path = category_path(list_channels_input.category);
    // Get any channels on this path
    let links = path.children()?;
    let mut channels = Vec::with_capacity(links.len());

    let mut channel_data: Vec<EntryHash> = Vec::new();
    // For each channel get the channel info links and choose the latest
    for target in links.into_iter().map(|link| link.target) {
        // Path links have their full path as the tag so
        // we don't need to get_links on the child.
        // The tag can be turned into the channel path
        // let channel_path = Path::try_from(&tag)?;

        // // Turn the channel path into the channel
        // let channel = Channel::try_from(&channel_path)?;

        // Get any channel info links on this channel
        let channel_info = get_links(target, Some(ChannelInfoTag::tag()))?;

        // Find the latest
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

        // If there is none we will skip this channel
        let latest_info: EntryHash = match latest_info.and_then(|l| l.target.into_entry_hash()) {
            Some(h) => h,
            None => continue,
        };

        channel_data.push(latest_info);
    }
    let chan_results_input = channel_data
        .into_iter()
        .map(|t| GetInput::new(t.into(), GetOptions::default()))
        .collect();
    let all_channel_results_elements = HDK.with(|hdk| hdk.borrow().get(chan_results_input))?;
    // Get the actual channel info entry
    for ele in all_channel_results_elements.into_iter() {
        if let Some(element) = ele {
            if let Some(info) = element.into_inner().1.to_app_option::<ChannelInfo>()? {
                // Turn the info into Channel
                channels.push(ChannelData {
                    entry: Channel {
                        category: info.category.clone(),
                        uuid: info.uuid.clone(),
                    },
                    info,
                })
            }
        }
    }

    // Return all the channels data to the UI
    Ok(channels.into())
}

// Note: This function can get very heavy
pub(crate) fn channel_stats(list_channels_input: ChannelListInput) -> ChatResult<(usize, usize)> {
    let channel_path = category_path(list_channels_input.category);

    let channel_links = channel_path.children()?;
    Ok((channel_links.len(), 0))
}
