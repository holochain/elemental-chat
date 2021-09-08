use std::time::Duration;

use hdk::prelude::*;
use timestamp::Timestamp;

use crate::error::ChatResult;

/// Get a local header from your chain
pub(crate) fn get_local_header(header_hash: &HeaderHash) -> ChatResult<Option<Header>> {
    // Get the latest chain header
    // Query iterates backwards so index 0 is the latest.
    let header = query(QueryFilter::new())?.into_iter().find_map(|el| {
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
pub(crate) fn to_date(ts: Timestamp) -> chrono::DateTime<chrono::Utc> {
    match chrono::DateTime::<chrono::Utc>::try_from(ts) {
        // docs suggest that this can fail and must be handled. however, it
        // seems unrecoverable.
        // https://github.com/holochain/holochain/blob/813d3ea68d35276ebd7ea0282c3b74e0460c46d3/crates/holochain_zome_types/src/timestamp.rs#L106-L108
        Err(err) => panic!("to_date: timestamp conversion error: {}", err),
        Ok(dt) => dt,
    }
}
