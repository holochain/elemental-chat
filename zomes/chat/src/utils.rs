use hdk3::prelude::*;

use crate::error::ChatResult;

pub(crate) fn get_local_header(header_hash: &HeaderHash) -> ChatResult<Option<HeaderHashed>> {
    // Get the latest chain header
    // Query iterates backwards so index 0 is the latest.
    let header = query!(QueryFilter::new())?
        .0
        .into_iter()
        .find(|h| h.as_hash() == header_hash);
    Ok(header)
}
