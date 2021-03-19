use hdk::prelude::*;
use crate::error::{ChatResult, ChatError};
use crate::message::Message;

/// Validate joining code from the membrane_proof
pub(crate) fn joining_code() -> ChatResult<()> {
    let filter = QueryFilter::new();
    let filter = filter.sequence_range(1..2);
    let query_result: Vec<Element> = query(filter)?;
    let holo_agent = AgentPubKey::try_from("uhCAkfzycXcycd-OS6HQHvhTgeDVjlkFdE2-XHz-f_AC_5xelQX1N").unwrap();

    match query_result[0].signed_header().header() {
        Header::AgentValidationPkg(pkg) => {
            match &pkg.membrane_proof {
                Some(mem_proof) => {
                    let joining_code: Element = Element::try_from(mem_proof.clone())?;
                    debug!("Joining code provided: {:?}", joining_code);

                    let _signature = joining_code.signature().clone();
                    let author = joining_code.header().author().clone();

                    if author == holo_agent {
                        debug!("Joining code valadated");
                        Ok(())
                    } else {
                        debug!("Joining code validation failed");
                        Err(ChatError::InitFailure)
                    }

                    // if verify_signature(holo_agent.clone(), signature, SerializedBytes::try_from(joining_code.entry().clone())?)? {
                    //     debug!("Joining code valadated");
                    //     Ok(())
                    // } else {
                    //     debug!("Joining code validation failed");
                    //     Err(ChatError::InitFailure)
                    // }
                }
                None => Err(ChatError::InitFailure)
            }
        },
        _ => Err(ChatError::InitFailure)
    }
}

pub(crate) fn common_validatation(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    let element = data.element;
    let entry = element.into_inner().1;
    let entry = match entry {
        ElementEntry::Present(e) => e,
        _ => return Ok(ValidateCallbackResult::Valid),
    };
    if let Entry::Agent(_) = entry {
        return Ok(ValidateCallbackResult::Valid);
    }
    Ok(match Message::try_from(&entry) {
        Ok(message) => {
            if message.content.len() <= 1024 {
                ValidateCallbackResult::Valid
            } else {
                ValidateCallbackResult::Invalid("Message too long".to_string())
            }
        }
        _ => ValidateCallbackResult::Valid,
    })
}
