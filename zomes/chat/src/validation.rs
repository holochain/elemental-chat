use crate::message::Message;
use hdk::prelude::*;

pub fn __validate_create_entry(entry: Entry) -> ExternResult<ValidateCallbackResult> {
    match entry {
        Entry::App(_) => match entry.try_into() {
            Ok(Message { content, .. }) => {
                if content.len() <= 1024 {
                    Ok(ValidateCallbackResult::Valid)
                } else {
                    Ok(ValidateCallbackResult::Invalid(
                        "Message too long".to_string(),
                    ))
                }
            }
            _ => Ok(ValidateCallbackResult::Valid),
        },
        _ => Ok(ValidateCallbackResult::Valid),
    }
}
