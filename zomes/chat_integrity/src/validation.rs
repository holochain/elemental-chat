use super::*;
use holochain_deterministic_integrity::prelude::*;

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

#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op {
        Op::StoreEntry { entry, .. } => validation::__validate_create_entry(entry),
        _ => Ok(ValidateCallbackResult::Valid),
    }
}
