//! TODO: Document how to use this crate in general
//!
//!
//!
use crate::{error::ChatResult, ChatError};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use hdk::prelude::*;
use std::cmp;

pub fn get_previous_hour(time: Timestamp) -> Result<Timestamp, TimestampError> {
    time - std::time::Duration::from_secs(60 * 60)
}

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
            // let earliest_seen_child_segment =
            //     last_segment_from_path(&earliest_seen_child_path).unwrap();
            let children = current_search_path.children()?;
            // This was just an optimization which we are going to loose
            // children.retain(|child_link| {
            //     link_is_earlier(child_link, earliest_seen_child_segment).unwrap_or(false)
            // });
            append_message_links_recursive(children, &mut links, target_count, depth)?;
        }

        earliest_seen_child_path = current_search_path;
        current_search_path = earliest_seen_child_path.parent().unwrap();
        depth += 1;
    }
    let mut clean_links_list = Vec::new();
    links.iter().for_each(|l| {
        if !clean_links_list.iter().any(|ll| ll == l) {
            clean_links_list.push(l.clone())
        }
    });

    Ok(clean_links_list)
}

// fn get_path_entry(hash: EntryHash) -> ChatResult<PathEntry> {
//     match get_details(hash, GetOptions::default())? {
//         Some(Details::Entry(EntryDetails { entry, .. })) => {
//             match PathEntry::try_from(entry.clone()) {
//                 Ok(e) => Ok(e),
//                 Err(_) => Err(ChatError::Generic(
//                     "Could not convert get_links result to requested type",
//                 )),
//             }
//         }
//         _ => Err(ChatError::Generic(
//             "get_details did not return an app entry",
//         )),
//     }
// }

fn append_message_links_recursive(
    mut children: Vec<Link>,
    links: &mut Vec<Link>,
    target_count: usize,
    depth: u8,
) -> ChatResult<()> {
    children.sort_unstable_by_key(|grandchild_link| cmp::Reverse(grandchild_link.timestamp));
    for child_link in children {
        if depth == 0 {
            let mut message_links = get_links(child_link.target, None)?;
            links.append(&mut message_links);
        } else {
            // let grandchildren = get_path_entry(child_link.target)?; //current_search_path.children()?;
            let grandchildren = get_links(child_link.target, None)?;
            append_message_links_recursive(grandchildren, links, target_count, depth - 1)?;
        }
        if links.len() >= target_count {
            break;
        }
    }

    Ok(())
}

// fn link_is_earlier(link: &Link, earlier_than: i64) -> ChatResult<bool> {
//     let path = Path::try_from(&link.tag)?;
//     let segment = last_segment_from_path(&path)?;
//     Ok(segment < earlier_than)
// }

pub fn last_segment_from_path(path: &Path) -> ChatResult<i64> {
    let component = path.as_ref().last().ok_or(ChatError::InvalidBatchingPath)?;
    let bytes: [u8; 8] = component
        .as_ref()
        .try_into()
        .map_err(|_| ChatError::InvalidBatchingPath)?;
    Ok(i64::from_be_bytes(bytes))
}

/// Add the message from the Date type to this path
pub fn timestamp_into_path(path: Path, time: Timestamp) -> ChatResult<Path> {
    let (ms, ns) = time.as_seconds_and_nanos();
    let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ms, ns), Utc);
    let mut components: Vec<_> = path.into();

    components.push(i64::from(time.year()).to_be_bytes().to_vec().into());
    components.push(i64::from(time.month()).to_be_bytes().to_vec().into());
    components.push(i64::from(time.day()).to_be_bytes().to_vec().into());
    // DEV_MODE: This can be updated to sec() for testing
    components.push(i64::from(time.hour()).to_be_bytes().to_vec().into());
    Ok(components.into())
}

pub fn depth_timestamp_into_path(path: Path, time: Timestamp, depth: i64) -> ChatResult<Path> {
    let (ms, ns) = time.as_seconds_and_nanos();
    let time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ms, ns), Utc);
    let mut components: Vec<_> = path.into();
    if depth >= 0 {
        components.push(i64::from(time.year()).to_be_bytes().to_vec().into());
    }
    if depth >= 1 {
        components.push(i64::from(time.month()).to_be_bytes().to_vec().into());
    }
    if depth >= 2 {
        components.push(i64::from(time.day()).to_be_bytes().to_vec().into());
    }
    if depth >= 3 {
        // DEV_MODE: This can be updated to sec() for testing
        components.push(i64::from(time.hour()).to_be_bytes().to_vec().into());
    }
    Ok(components.into())
}
