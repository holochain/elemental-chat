use std::time::Duration;

use hdk3::prelude::*;
use timestamp::Timestamp;

use crate::error::ChatResult;

/// Get a local header from your chain
pub(crate) fn get_local_header(header_hash: &HeaderHash) -> ChatResult<Option<Header>> {
    // Get the latest chain header
    // Query iterates backwards so index 0 is the latest.
    let header = query!(QueryFilter::new())?.0.into_iter().find_map(|el| {
        if el.header_address() == header_hash {
            Some(
                el.into_inner()
                    .0
                    .into_header_and_signature()
                    .0
                    .into_content(),
            )
        } else {
            None
        }
    });
    Ok(header)
}

/// Turns a unix timestamp into a holochain Timestamp
pub(crate) fn to_timestamp(duration: Duration) -> Timestamp {
    Timestamp(duration.as_secs() as i64, duration.subsec_nanos())
}

/// Turns a unix timestamp into a Date
pub(crate) fn to_date(duration: Duration) -> chrono::Date<chrono::Utc> {
    use chrono::{DateTime, NaiveDateTime, Utc};
    let s = duration.as_secs() as i64;
    let n = duration.subsec_nanos();
    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(s, n), Utc).date()
}
