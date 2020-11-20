use crate::{
    channel::{Channel, ChannelInput},
    entries::message::SignalMessageData,
    error::{ChatError, ChatResult},
    utils::{to_date, to_timestamp},
};
use hdk3::prelude::*;
use link::Link;

use super::{ChannelData, ChannelInfo, ChannelInfoTag, ChannelList, ChannelListInput};

/// Create a new channel
/// This effectively just stores channel info on the
/// path that is `category:channel_id`
pub(crate) fn create_channel(channel_input: ChannelInput) -> ChatResult<ChannelData> {
    let ChannelInput { name, channel } = channel_input;

    // Create the path for this channel
    let path: Path = channel.clone().into();
    path.ensure()?;

    // Create the channel info
    let info = ChannelInfo {
        // This agent
        created_by: agent_info()?.agent_initial_pubkey,
        // Right now
        created_at: to_timestamp(sys_time()?),
        name,
    };

    // Commit the channel info
    create_entry(&info)?;
    let info_hash = hash_entry(&info)?;

    // link the channel info to the path
    create_link(path.hash()?, info_hash, ChannelInfoTag::tag())?;

    // Return the channel and the info for the UI
    Ok(ChannelData::new(channel, info))
}

pub(crate) fn list_channels(list_channels_input: ChannelListInput) -> ChatResult<ChannelList> {
    // Get the category path
    let path = Path::from(list_channels_input.category);

    // Get any channels on this path
    let links = path.children()?.into_inner();
    let mut channels = Vec::with_capacity(links.len());

    // For each channel get the channel info links and choose the latest
    for tag in links.into_iter().map(|link| link.tag) {
        // Path links have their full path as the tag so
        // we don't need to get_links on the child.
        // The tag can be turned into the channel path
        let channel_path = Path::try_from(&tag)?;

        // Turn the channel path into the channel
        let channel = Channel::try_from(&channel_path)?;

        // Check if our agent key is active on this path and
        // add it if it's not
        add_chatter(channel.chatters_path())?;

        // Get any channel info links on this channel
        let channel_info =
            get_links(channel_path.hash()?, Some(ChannelInfoTag::tag()))?.into_inner();

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
        let latest_info = match latest_info {
            Some(l) => l,
            None => continue,
        };

        // Get the actual channel info entry
        if let Some(element) = get(latest_info.target, GetOptions)? {
            if let Some(info) = element.into_inner().1.to_app_option()? {
                // Construct the channel data from the channel and info
                channels.push(ChannelData { channel, info });
            }
        }
    }
    // Return all the channels data to the UI
    Ok(channels.into())
}

pub(crate) fn signal_users_on_channel(signal_message_data: SignalMessageData) -> ChatResult<()> {
    let path: Path = signal_message_data.channel.chatters_path();
    let hour_path = add_current_hour_path(path.clone())?;
    signal_hour(hour_path, signal_message_data.clone())?;
    let hour_path = add_current_hour_minus_n_path(path, 1)?;
    signal_hour(hour_path, signal_message_data)?;
    Ok(())
}

fn signal_hour(hour_path: Path, signal_message_data: SignalMessageData) -> ChatResult<()> {
    let chatters = get_links(hour_path.hash()?, None)?.into_inner();
    for tag in chatters.into_iter().map(|l| l.tag) {
        let agent = tag_to_agent(tag)?;
        call_remote(
            agent,
            "chat".to_string().into(),
            "new_message_signal".to_string().into(),
            None,
            &signal_message_data,
        )?;
    }
    Ok(())
}

fn add_chatter(path: Path) -> ChatResult<()> {
    let agent = agent_info()?.agent_latest_pubkey;
    let agent_tag = agent_to_tag(&agent);

    let hour_path = add_current_hour_path(path.clone())?;
    let my_chatter = get_links(hour_path.hash()?, Some(agent_tag.clone()))?.into_inner();
    if my_chatter.is_empty() {
        create_link(hour_path.hash()?, agent.into(), agent_tag)?;
    }

    Ok(())
}

fn agent_to_tag(agent: &AgentPubKey) -> LinkTag {
    let agent_tag: &[u8] = agent.as_ref();
    LinkTag::new(agent_tag)
}

fn tag_to_agent(tag: LinkTag) -> ChatResult<AgentPubKey> {
    Ok(AgentPubKey::from_raw_39(tag.0).map_err(|_| ChatError::AgentTag)?)
}

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
