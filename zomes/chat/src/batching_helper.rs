//! TODO: Document how to use this crate in general
//!
//!
//!
//use std::thread::current;

use crate::{error::ChatResult, ChatError};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use hdk::{hash_path::path::Component, prelude::*};
//use std::cmp;
use std::convert::TryInto;

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
    let mut done: bool;

    // create the skeleton of the current siblings we are searching in the tree by finding the first existing leaf
    // with placeholders for lazy loading the actual siblings in case we get what we want where we are
    let mut current_siblings = SearchState::new(search_path.clone())?;

    done = !current_siblings.find_existing_leaf()?;

    debug!("cur_sibs {:?}", current_siblings);

    while !done {
        debug!("search_path {:?}", pretty_path(&current_siblings.path));
        let l = &mut get_links(current_siblings.path.path_entry_hash()?, None)?;
        debug!("appending {} {:?}", l.len(), l);
        links.append(l);
        if links.len() < target_count {
            done = !current_siblings.find_previous_leaf()?;
        } else {
            done = true
        }
    }

    Ok(links)
}

// siblings is an ordered list the year/month/day/hours that are returned by calling path.children
type Siblings = Vec<i64>;
#[derive(Debug)]
pub struct Level {
    maybe_sibs: Option<Siblings>,
    current: i64, // current sib
}
#[derive(Debug)]
pub struct SearchState {
    path: Path,
    level: usize,
    levels: Vec<Level>,
}
const TREE_DEPTH: usize = 6;
const TREE_TOP: usize = 2;

impl SearchState {
    pub fn new(path: Path) -> ChatResult<Self> {
        // check that our path is starting at a leaf
        if path.as_ref().len() != TREE_DEPTH {
            return Err(ChatError::InvalidBatchingPath.into());
        }

        let levels = path
            .as_ref()
            .into_iter()
            .skip(TREE_TOP)
            .filter_map(|c| match compontent_to_i64(c) {
                Ok(i) => Some(Level {
                    maybe_sibs: None,
                    current: i,
                }),
                Err(_) => None,
            })
            .collect();

        Ok(SearchState {
            path,
            level: TREE_DEPTH - TREE_TOP - 1, // level is zero based index
            levels,
        })
    }
    pub fn get_level(&self) -> &Level {
        //        debug!("get level of {:?} @ {}", self.levels, self.level);
        &self.levels[self.level]
    }
    pub fn get_sibs(&mut self) -> &Option<Siblings> {
        &self.get_level().maybe_sibs
    }
    pub fn get_current(&self) -> i64 {
        self.get_level().current
    }
    pub fn set_sibs(&mut self, sibs: Siblings) {
        debug!("setting: {:?} at level: {}", sibs, self.level);
        let current = sibs[sibs.len() - 1];
        self.levels[self.level].maybe_sibs = Some(sibs);
        self.levels[self.level].current = current;
    }
    pub fn up(&mut self) {
        self.level -= 1;
    }
    pub fn down(&mut self) {
        self.level += 1;
    }
    pub fn at_bottom(&self) -> bool {
        self.level == TREE_DEPTH - TREE_TOP - 1
    }

    /// find an existing leaf on the time tree starting from the search path given, and returning the path found,
    /// as well as the search state found time path portions while searching for that first path
    pub fn find_existing_leaf(self: &mut SearchState) -> Result<bool, WasmError> {
        let mut search_path = self.path.clone();
        debug!("starting with {:?}", self);
        debug!("search_path {:?}", pretty_path(&search_path));

        // we only want to walk the date part of the path which is the last 4 components.
        let mut level = TREE_DEPTH;

        // walk up the tree till we find something that exists
        while !search_path.exists()? && level >= TREE_TOP {
            level -= 1;
            self.up();
            match search_path.parent() {
                None => {
                    // if there's no parent (shouldn't happen), then there's nothing on this whole path so we can return that there is no such path
                    return Ok(false);
                }
                Some(parent) => {
                    search_path = parent;
                }
            }
        }

        debug!(
            "search_path {}, level: {}",
            pretty_path(&search_path),
            level
        );

        // if we got above the top of the tree, then there's nothing on this whole tree so we can return None
        if search_path.as_ref().len() == TREE_TOP - 1 {
            return Ok(false);
        }
        debug!("walked up search_path {}", pretty_path(&search_path));

        // walk back down the tree storing the sibling info as we go
        while search_path.as_ref().len() < TREE_DEPTH {
            self.down();
            debug!(
                "finding sibs {} @ {}",
                pretty_path(&search_path),
                self.level
            );
            let sibs = find_children(&search_path)?;
            debug!("sibs @ LEVEL {:?} {}", sibs, self.level);
            // set the current sib to the last one
            if sibs.len() == 0 {
                break;
            }
            let last_sib = sibs[sibs.len() - 1];
            self.set_sibs(sibs);
            search_path.append_component(last_sib.to_be_bytes().to_vec().into());
            debug!("search path now: {}", pretty_path(&search_path))
        }
        debug!("ending with {:?}", self);
        debug!("ending search_path {}", pretty_path(&search_path));
        self.path = search_path;
        Ok(true)
    }

    /// looks for the previous leaf, perhaps by searching up and back down the tree
    /// Assumes that the current search_path does exist, but not necessarily that any
    /// siblings were loaded into the SearchState at that part of the tree
    pub fn find_previous_leaf(&mut self) -> Result<bool, WasmError> {
        debug!("finding previous leaf");

        loop {
            let found;
            // if we hadn't done a siblings search before for this level of the tree, do so now
            if self.get_sibs().is_none() {
                debug!("sibs is None, searching...");
                let x = self.find_siblings()?;
                self.set_sibs(x.clone());
                debug!("siblings found: {:?}", x);
                found = true;
            } else {
                // move the the previous sib at this level if possible
                found = self.previous_sib();
                if !found {
                    // if there was no previous sib, then check to see
                    // if we were are allready at the top.  If so there's no previous leaf
                    if self.level == 0 {
                        debug!("top and no sibs");
                        return Ok(false);
                    }
                    // clear this level and go up
                    self.levels[self.level].maybe_sibs = None;
                    self.level -= 1;
                    debug!("going up to {}", self.level);
                }
                debug!("previous_sib result: {:?}", self.levels);
            }
            self.set_path();
            if found {
                // if this is the bottom level, the we have found a leaf
                if self.at_bottom() {
                    debug!("found leaf setting path: {}", pretty_path(&self.path));
                    return Ok(true);
                }
                // otherwise go down
                self.level += 1;
                debug!("going down to {}", self.level);
            }
        }
    }

    fn set_path(&mut self) {
        let mut components: Vec<Component> = vec![];
        components.push(self.path.as_ref()[0].clone());
        components.push(self.path.as_ref()[1].clone());
        for level in &self.levels {
            components.push(level.current.to_be_bytes().to_vec().into());
        }
        self.path = Path::from(components);
        debug!("level: {} levels: {:?}", self.level, self.levels);
        debug!("path: {}", pretty_path(&self.path));
    }

    fn previous_sib(&mut self) -> bool {
        let current = self.get_current();
        if let Some(ref mut sibs) = self.levels[self.level].maybe_sibs {
            sibs.retain(|sib| *sib < current);
            //debug!("retained: {:?}", self.get_sibs());
            if let Some(last) = sibs.last() {
                self.levels[self.level].current = *last;
                return true;
            }
        }
        return false;
    }

    fn find_siblings(&self) -> ChatResult<Vec<i64>> {
        let x = self.path.as_ref()[0..(self.level + TREE_TOP)].to_vec();
        let p = Path::from(x);
        debug!("find_sibs path {}", pretty_path(&p));
        find_children(&p)
    }
}

fn find_children(path: &Path) -> ChatResult<Vec<i64>> {
    let sibs: Result<Vec<_>, _> = path
        .children_paths()?
        .into_iter()
        .map(last_segment_from_path)
        .collect();
    let mut r = sibs?;
    r.sort();
    Ok(r)
}

pub fn pretty_path(path: &Path) -> String {
    let p: Vec<String> = path
        .as_ref()
        .into_iter()
        .skip(TREE_TOP)
        .map(|c| compontent_to_i64(c).unwrap().to_string())
        .collect();
    p.join(".")
}

pub fn compontent_to_i64(component: &Component) -> ChatResult<i64> {
    let bytes: [u8; 8] = component
        .as_ref()
        .try_into()
        .map_err(|_| ChatError::InvalidBatchingCompontent)?;
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

pub fn last_segment_from_path(path: Path) -> ChatResult<i64> {
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
    components.push(i64::from(time.hour()).to_be_bytes().to_vec().into());
    Ok(components.into())
}
