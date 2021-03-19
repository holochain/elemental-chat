use hdk::prelude::*;
use crate::message::Message;

/// Validate joining code from the membrane_proof
pub(crate) fn joining_code(element: Element) -> ExternResult<ValidateCallbackResult> {
    let holo_agent = AgentPubKey::try_from("uhCAkfzycXcycd-OS6HQHvhTgeDVjlkFdE2-XHz-f_AC_5xelQX1N").unwrap();

    match element.signed_header().header() {
        Header::AgentValidationPkg(pkg) => {
            match &pkg.membrane_proof {
                Some(mem_proof) => {
                    let mem_proof: Element = Element::try_from(mem_proof.clone())?;
                    debug!("Joining code provided: {:?}", mem_proof);

                    let author = mem_proof.header().author().clone();

                    if author != holo_agent {
                        debug!("Joining code validation failed");
                        return Ok(ValidateCallbackResult::Invalid(format!("Joining code created by incorect admin {:?}", author)))
                    }

                    let _signature = mem_proof.signature().clone();
                    // if verify_signature(holo_agent.clone(), signature, SerializedBytes::try_from(joining_code.entry().clone())?)? {
                    //     debug!("Joining code valadated");
                    //     Ok(())
                    // } else {
                    //     debug!("Joining code validation failed");
                    //     Ok(ValidateCallbackResult::Invalid("Unable to validate".to_string()))
                    // }
                    debug!("Joining code create by the right agent");
                    return Ok(ValidateCallbackResult::Valid)
                }
                None => Ok(ValidateCallbackResult::Invalid("Unable to validate".to_string()))
            }
        },
        _ => Ok(ValidateCallbackResult::Invalid("Unable to validate".to_string()))
    }
}

pub(crate) fn common_validatation(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    let element = data.element.clone();
    let entry = element.into_inner().1;
    let entry = match entry {
        ElementEntry::Present(e) => e,
        _ => return Ok(ValidateCallbackResult::Valid),
    };
    if let Entry::Agent(_) = entry {
        match data.element.header().prev_header() {
            Some(header) => {
                match get(header.clone(), GetOptions::default())? {
                    Some(element_pkg) => {
                        return joining_code(element_pkg)
                    },
                    None => return Ok(ValidateCallbackResult::Invalid("Unable to validate".to_string()))
                }
            },
            None => return Ok(ValidateCallbackResult::Invalid("Impossible state".to_string()))
        }
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
