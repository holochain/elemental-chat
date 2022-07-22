use hdk::prelude::*;
use timestamp::Timestamp;

use crate::error::ChatResult;

/// Get a local action from your chain
pub(crate) fn get_local_action(action_hash: &ActionHash) -> ChatResult<Option<Action>> {
    // Get the latest chain action
    // Query iterates backwards so index 0 is the latest.
    let action = query(QueryFilter::new())?.into_iter().find_map(|el| {
        if el.action_address() == action_hash {
            Some(el.into_inner().0.into_inner().0.into_content())
        } else {
            None
        }
    });
    Ok(action)
}

/// Turns a unix timestamp into a Date
pub(crate) fn to_date(timestamp: Timestamp) -> chrono::DateTime<chrono::Utc> {
    timestamp.try_into().unwrap()
}
