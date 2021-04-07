use hdk::prelude::*;
use crate::message::Message;

/// This is the current structure of the payload the holo signs
#[hdk_entry(id = "joining_code_payload")]
#[derive(Clone)]
struct JoiningCodePayload {
    role: String,
    record_locator: String
}

/// Validate joining code from the membrane_proof
pub(crate) fn joining_code(element: Element) -> ExternResult<ValidateCallbackResult> {
    // This is a hard coded holo agent public key
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
                        return Ok(ValidateCallbackResult::Invalid(format!("Joining code invalid: unexpected author ({:?})", author)))
                    }

                    // let signature = mem_proof.signature().clone();
                    if let ElementEntry::Present(_entry) = mem_proof.entry() {
                        if *mem_proof.header().author() != holo_agent {
                            debug!("Joining code not created by holo_agent");
                            return Ok(ValidateCallbackResult::Invalid("Joining code invalid: incorrect holo agent".to_string()))
                        }

                        debug!("Joining code validated without checking signature");
                        return Ok(ValidateCallbackResult::Valid)
/*
                        let jcp = JoiningCodePayload::try_from(entry.clone())?;
                        let jcp = element.into_inner().1;
                        if verify_signature(holo_agent.clone(), signature, SerializedBytes::try_from(jcp.clone())?)? {
                            debug!("Joining code validated");
                            return Ok(ValidateCallbackResult::Valid)
                        } else {
                            debug!("Joining code validation failed");
                            return Ok(ValidateCallbackResult::Invalid("Joining code invalid: incorrect signature".to_string()))
                        }*/
                    } else {
                        return Ok(ValidateCallbackResult::Invalid("Joining code invalid payload".to_string()));
                    }
                }
                None => Ok(ValidateCallbackResult::Invalid("No membrane proof found".to_string()))
            }
        },
        _ => Ok(ValidateCallbackResult::Invalid("No Agent Validation Pkg found".to_string()))
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
                    None => return Ok(ValidateCallbackResult::Invalid("Agent validation failed: missing element".to_string()))
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
