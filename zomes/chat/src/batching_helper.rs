//! TODO: Document how to use this crate in general
//!
//!
//!
use crate::{error::ChatResult, ChatError};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use hdk::{prelude::*, hash_path::path::Component};
//use std::cmp;

// siblings is an ordered list the year/month/day/hours that are returned by calling path.children
type Siblings = Vec<i64>;

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

    let _root_path_length = channel.as_ref().len();
    let newest_included_hour_path = timestamp_into_path(channel, newest_included_hour)?;

    let /*mut */ search_path = newest_included_hour_path.clone();
    let mut done = false;

    // create the skeleton of the current siblings we are searching in the tree
    let mut _current_siblings: Vec<(Option<Siblings>,i64)> = search_path.as_ref().into_iter().map(|c| {
        (None, match compontent_to_i64(c) {Ok(i) => i, Err(_) => 0})}).collect();
    while links.len() < target_count && !done {
        if search_path.exists()? {
            links.append(&mut get_links(newest_included_hour_path.path_entry_hash()?, None)?);
        } else {/*
            match previous_sibling(search_path) {
                Some(path) => search_path = path,
                None => match previous_cousin(search_path) {
                    Some(path) => search_path = path,
                    None => done = true
                }
            }*/
            done = true
        }
    }
/* 
    fn siblings(path: Path) -> Result<Option<Path>, WasmError> {
        match path.parent() {
            None => Ok(None),
            Some(parent) => {
                let siblings = get_links(parent.path_entry_hash()?, None)?;
                // TODO examine siblings to find which is previous to path
                Ok(None)
            }
        }      
    }
*/
    /* 
    let mut earliest_seen_child_path = newest_included_hour_path;
    let mut current_search_path = earliest_seen_child_path.parent().unwrap();
    let mut depth = 0;
    while links.len() < target_count && current_search_path.as_ref().len() >= root_path_length {
        if current_search_path.exists()? {
            let earliest_seen_child_segment =
                last_segment_from_path(&earliest_seen_child_path).unwrap();
            let mut children = current_search_path.children()?;
            children.retain(|child_link| {
                link_is_earlier(child_link, earliest_seen_child_segment).unwrap_or(false)
            });
            append_message_links_recursive(children, &mut links, target_count, depth)?;
        }

        earliest_seen_child_path = current_search_path;
        current_search_path = earliest_seen_child_path.parent().unwrap();
        depth += 1;
    }
*/
    Ok(links)
}
/* 
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
            let path = Path::try_from(&child_link.tag)?;
            let grandchildren = path.children()?;
            append_message_links_recursive(grandchildren, links, target_count, depth - 1)?;
        }
        if links.len() >= target_count {
            break;
        }
    }

    Ok(())
}

fn link_is_earlier(link: &Link, earlier_than: i64) -> ChatResult<bool> {
    let path = Path::try_from(&link.tag)?;
    let segment = last_segment_from_path(&path)?;
    Ok(segment < earlier_than)
} */

pub fn compontent_to_i64(component: &Component) -> ChatResult<i64> {
    let bytes: [u8; 8] = component
        .as_ref()
        .try_into()
        .map_err(|_| ChatError::InvalidBatchingPath)?;
    Ok(i64::from_be_bytes(bytes))
}

pub fn last_segment_from_path(path: &Path) -> ChatResult<i64> {
    let component = path.as_ref().last().ok_or(ChatError::InvalidBatchingPath)?;
    compontent_to_i64(component)
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
