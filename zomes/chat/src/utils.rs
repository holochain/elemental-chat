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

/// Turns a unix timestamp into a Date
pub(crate) fn to_date(timestamp: Timestamp) -> chrono::DateTime<chrono::Utc> {
    timestamp.try_into().unwrap()
}
