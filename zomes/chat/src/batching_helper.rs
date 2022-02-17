//! TODO: Document how to use this crate in general
//!
//!
//!
//use std::thread::current;

use crate::{error::ChatResult, ChatError};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use hdk::{prelude::*, hash_path::path::Component};
//use std::cmp;
use std::convert::TryInto;

// siblings is an ordered list the year/month/day/hours paths that are returned by calling path.children 
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

    let search_path = newest_included_hour_path.clone();
    let mut done = false;

    let (search_path, _current_siblings) = find_existing_leaf(search_path)?.ok_or(ChatError::InvalidBatchingPath)?;
    

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


    Ok(links)
}

type SearchState = Vec<(Option<Siblings>,i64)>;

pub fn find_existing_leaf(mut search_path: Path) -> Result<Option<(Path, SearchState)>, WasmError>{
    // create the skeleton of the current siblings we are searching in the tree by finding the first existing leaf
    // with placeholders for lazy loading the actual siblings in case we get what we want where we are
    let mut current_siblings: SearchState = search_path.as_ref().into_iter().map(|c| {
        (None, match compontent_to_i64(c) {Ok(i) => i, Err(_) => 0})}).collect();
    
    if current_siblings.len() != 4 {
        debug!("Path isn't of correct depth!");
        return Err(ChatError::InvalidBatchingPath.into())
    }

    // walk up the tree till we find something that exists
    while !search_path.exists()? {
        match search_path.parent() {
            None =>
                // if there's no parent, then there's nothing on this whole path so we can return that there is no such path
                return Ok(None),
            Some(parent) => {
                search_path = parent;
            }
        }
    }

    // walk back down the tree storing the sibling info as we go
    while search_path.as_ref().len() < 4 {
        let children = search_path.children_paths()?;
        let sibs: Vec<i64>  = children.into_iter()

        // filter out any parts that are after our current path
        .filter(|path| {
            let level = search_path.as_ref().len()-1;
            let component: &Component = &path.as_ref()[level];
            match compontent_to_i64(component) {
                Err(_) => false,
                Ok(i) => {
                    let (_, current) = current_siblings[level];
                    i <= current
                }
            }
        })
        .map(|path| {
            match path.leaf() {
                None => -1 as i64,  // TODO FIXME?
                Some(component) => match compontent_to_i64(component) {Err(_)=> -1,Ok(i)=> i}
            }
        }).collect();
        // set the current sib to the last one
        let last_sib = sibs[sibs.len()-1];
        current_siblings[search_path.as_ref().len()-1] = (Some(sibs), last_sib);
        search_path.append_component(last_sib.to_be_bytes().to_vec().into());
    }

    Ok(Some((search_path,current_siblings)))
}

pub fn compontent_to_i64(component: &Component) -> ChatResult<i64> {
    let bytes: [u8; 8] = component
        .as_ref()
        .try_into()
        .map_err(|_| ChatError::InvalidBatchingPath)?;
    Ok(i64::from_be_bytes(bytes))
}

/*
// TODO there has to be a better way...
fn pop8(barry: &[u8]) -> &[u8; 8] {
    barry.try_into().expect("slice with incorrect length")
}

pub fn tag_to_i64(tag: &LinkTag) -> ChatResult<i64> {
    let tag_vec = tag.as_ref();
    if tag_vec.len() != 8 {
        return Err(ChatError::InvalidBatchingPath)
    }
    let bytes: [u8; 8] = *pop8(tag_vec);
    Ok(i64::from_be_bytes(bytes))
}*/

pub fn last_segment_from_path(path: &Path) -> ChatResult<i64> {
    let component = path.as_ref().last().ok_or(ChatError::InvalidBatchingPath)?;
    compontent_to_i64(component)
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
        depth += 1;    // create the skeleton of the current siblings we are searching in the tree by finding the first existing leaf
    // with placeholders for lazy loading the actual siblings in case we get what we want where we are
    let mut current_siblings: Vec<(Option<Siblings>,i64)> = search_path.as_ref().into_iter().map(|c| {
        (None, match compontent_to_i64(c) {Ok(i) => i, Err(_) => 0})}).collect();
    
    if current_siblings.len() != 4 {
        debug!("Path isn't of correct depth!");
        return Err(ChatError::InvalidBatchingPath.into())
    }

    // walk up the tree till we find something that exists
    let level =
    while (!search_path.exists?) {
        match search_path.parent() {
            None =>
                // if there's no parent, then there's nothing on this whole path so we can return an empty list
                return Ok(vec![]),
            Some(parent) => {
                let sibs = parent.children()?;

            }
    }

    }
*/