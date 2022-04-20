//! TODO: Document how to use this crate in general
//!
//!
//!
use crate::{error::ChatResult, ChatError};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use hdk::{hash_path::path::Component, prelude::*};
use std::cmp;

pub fn get_previous_hour(time: Timestamp) -> Result<Timestamp, TimestampError> {
    time - std::time::Duration::from_secs(60 * 60)
}

/// Returns at least `target_count` messages that are all earlier than `earliest_seen`.
///
/// Navigates a tree of timestamp-based links to find messages.
/// We used to link all the messages for a channel in the same place,
/// but it was too slow to load them, so we created this tree to reduce the work done per zome call.
pub fn get_message_links(
    channel: Path,
    earliest_seen: Option<Timestamp>,
    target_count: usize,
) -> Result<Vec<Link>, WasmError> {
    let newest_included_hour = if let Some(earliest_seen) = earliest_seen {
        if let Ok(hour) = earliest_seen - std::time::Duration::from_secs(60 * 60) {
            hour
        } else {
            return Ok(Vec::new());
        }
    } else {
        sys_time()?
    };

    let mut links = Vec::new();

    let root_path_length = channel.as_ref().len();
    let newest_included_hour_path = timestamp_into_path(channel, newest_included_hour)?;
    if newest_included_hour_path.exists()? {
        links.append(&mut get_links(
            newest_included_hour_path.path_entry_hash()?,
            None,
        )?);
    }

    let mut earliest_seen_child_path = newest_included_hour_path;
    let mut current_search_path = earliest_seen_child_path.parent().unwrap();
    let mut depth = 0;
    while links.len() < target_count && current_search_path.as_ref().len() >= root_path_length {
        if current_search_path.exists()? {
            let earliest_seen_child_segment =
                last_segment_from_path(&earliest_seen_child_path).unwrap();
            let children = current_search_path.children()?;

            let raw_children = children
                .iter()
                .map(|l| format!("{{ tag: {:?} timestamp: {:?} }}, ", l.tag, l.timestamp))
                .collect::<String>();
            let mut children = children
                .into_iter()
                .filter_map(|l| path_component_from_link(&l).ok().map(|c| (c, l))) // filter out non-path links
                .map(|(c, l)| Ok((segment_from_component(&c)?, l)))
                .collect::<Result<Vec<_>, ChatError>>()?;

            children.retain(|(segment, _)| *segment < earliest_seen_child_segment);

            let link_count_before = links.len();
            append_message_links_recursive(children, &mut links, target_count, depth)?;

            let links_added = links.get(link_count_before..).unwrap_or(&[]);
            debug!("batching: Finished including all descendants of node in tree (depth {:?} current_search_path {:?}).
            Raw children {:?}. Messages added {:?}", depth, current_search_path, raw_children, links_added);
        }

        earliest_seen_child_path = current_search_path;
        current_search_path = earliest_seen_child_path.parent().unwrap();
        depth += 1;
    }

    Ok(links)
}

fn append_message_links_recursive(
    mut children: Vec<(i32, Link)>,
    links: &mut Vec<Link>,
    target_count: usize,
    depth: u8,
) -> ChatResult<()> {
    children.sort_unstable_by_key(|(segment, _)| cmp::Reverse(*segment));
    for (_, link) in children {
        if depth == 0 {
            let mut message_links = get_links(link.target, None)?;
            links.append(&mut message_links);
        } else {
            let grandchildren = get_links(link.target, None)?;
            let grandchildren = grandchildren
                .into_iter()
                .filter_map(|l| path_component_from_link(&l).ok().map(|c| (c, l))) // filter out non-path links
                .map(|(c, l)| Ok((segment_from_component(&c)?, l)))
                .collect::<Result<Vec<_>, ChatError>>()?;
            append_message_links_recursive(grandchildren, links, target_count, depth - 1)?;
        }
        if links.len() >= target_count {
            break;
        }
    }

    Ok(())
}

fn path_component_from_link(link: &Link) -> Result<Component, SerializedBytesError> {
    SerializedBytes::from(UnsafeBytes::from(link.tag.clone().into_inner())).try_into()
}

fn segment_from_component(component: &Component) -> ChatResult<i32> {
    let bytes: [u8; 4] = component
        .as_ref()
        .try_into()
        .map_err(|_| ChatError::InvalidBatchingPath)?;
    Ok(i32::from_be_bytes(bytes))
}

pub fn last_segment_from_path(path: &Path) -> ChatResult<i32> {
    let component = path.leaf().ok_or(ChatError::InvalidBatchingPath)?;
    segment_from_component(component)
}

/// Add the message from the Date type to this path
pub fn timestamp_into_path(path: Path, time: Timestamp) -> ChatResult<Path> {
    let (ms, ns) = time.as_seconds_and_nanos();
    let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ms, ns), Utc);
    let mut components: Vec<_> = path.into();

    components.push((time.year() as i32).to_be_bytes().to_vec().into());
    components.push((time.month() as i32).to_be_bytes().to_vec().into());
    components.push((time.day() as i32).to_be_bytes().to_vec().into());
    // DEV_MODE: This can be updated to sec() for testing
    components.push((time.hour() as i32).to_be_bytes().to_vec().into());
    Ok(components.into())
}
