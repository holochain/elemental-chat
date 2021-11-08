//! TODO: Document how to use this crate in general
//!
//!
//!
use crate::error::ChatResult;
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, Timelike, Utc};
use hdk::prelude::*;
use std::ops::Sub;

///
pub fn get_batch_links(
    base: Path,
    earlier_than: Timestamp,
    target_count: usize,
) -> ChatResult<Vec<Link>> {
    let mut links: Vec<Link> = Vec::new();
    let mut next_path = timestamp_into_path(base.clone(), earlier_than, None)?;

    loop {
        debug!("Next path: {:?}", next_path.clone());
        // Confirm the path exists
        if next_path.exists()? {
            debug!("next path exists");
            // Get the actual hash we are going to pull the messages from
            let channel_entry_hash = next_path.hash()?;
            // Get the message links on this channel
            let nm = &mut get_links(channel_entry_hash.clone(), None)?.into_inner();
            let nm_check = nm.is_empty();
            links.append(nm);
            if (links.len() >= target_count) || nm_check {
                debug!("links.len() BREAK");
                break;
            }
        } else {
            debug!("next.path does not exists");
        }
        // set next path
        match get_next_path(&base, next_path)? {
            Some(p) => next_path = p,
            None => {
                debug!("Crawled through the entire tree");
                break;
            }
        }
    }
    Ok(links)
}

///
pub fn get_next_path(channel: &Path, last_seen: Path) -> ChatResult<Option<Path>> {
    let (ly, lm, l_day, mut l_hour) = path_spread(&last_seen)?;
    debug!("Crawling through year {} and month {}", ly, lm);
    // get base that starts with year and month
    let base_ym = get_year_month_path(&channel, &last_seen)?; // ROOT->Y->M
    let days = base_ym.children()?.into_inner();
    for tag in days.clone().into_iter().rev().map(|link| link.tag) {
        let day = Path::try_from(&tag)?; // ROOT->Y->M->D
        let dp: &Vec<_> = day.as_ref();
        debug!("Checking day {:?}", String::try_from(&dp[dp.len() - 1])?);
        // check if latest day is less than path day
        if format!("{}", l_day) >= String::try_from(&dp[dp.len() - 1])? {
            let hours = day.children()?.into_inner();
            for tag in hours.clone().into_iter().rev().map(|link| link.tag) {
                let hour_path = Path::try_from(&tag)?; // ROOT->Y->M->D->H
                let hp: &Vec<_> = hour_path.as_ref();
                let hour = String::try_from(&hp[hp.len() - 1])?;
                debug!("Checking hour:  {:?}", hour);
                if hour < format!("{}", l_hour) {
                    debug!("Next hour selected:  {:?}", hour);
                    return Ok(Some(hour_path));
                }
            }
            debug!("Resetting the hour to 23:00");
            l_hour = "23".to_string()
        }
    }

    Ok(None)
}

/// Add the message from the Date type to this path
pub fn timestamp_into_path(
    path: Path,
    time: Timestamp,
    interval: Option<Duration>, // Duration that will be subtracted from time to get a path for that time
) -> ChatResult<Path> {
    let (ms, ns) = time.as_seconds_and_nanos();
    let mut time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ms, ns), Utc);
    if let Some(i) = interval {
        time = time.sub(i);
    }
    let mut components: Vec<_> = path.into();

    components.push(format!("{}", time.year()).into());
    components.push(format!("{}", time.month()).into());
    components.push(format!("{}", time.day()).into());
    // DEV_MODE: This can be updated to sec() for testing
    components.push(format!("{}", time.hour()).into());
    Ok(components.into())
}
///
pub fn path_spread(p: &Path) -> ChatResult<(String, String, String, String)> {
    let p: &Vec<_> = p.as_ref();
    let l = p.len();
    Ok((
        String::try_from(&p[l - 4])?,
        String::try_from(&p[l - 3])?,
        String::try_from(&p[l - 2])?,
        String::try_from(&p[l - 1])?,
    ))
}
///
pub fn get_year_month_path(path: &Path, time_path: &Path) -> ChatResult<Path> {
    let mut components: Vec<_> = path.to_owned().into();
    let (y, m, _, _) = path_spread(time_path)?;
    components.push(y.into());
    components.push(m.into());
    Ok(components.into())
}
