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
                    let mem_proof = match Element::try_from(mem_proof.clone()) {
                        Ok(m) => m,
                        Err(e) => return Ok(ValidateCallbackResult::Invalid(format!("Joining code invalid: unable to deserialize into element ({:?})", e)))
                    };

                    debug!("Joining code provided: {:?}", mem_proof);

                    let author = mem_proof.header().author().clone();

                    if author != holo_agent {
                        debug!("Joining code validation failed");
                        return Ok(ValidateCallbackResult::Invalid(format!("Joining code invalid: unexpected author ({:?})", author)))
                    }

                    if let ElementEntry::Present(_entry) = mem_proof.entry() {
                        if *mem_proof.header().author() != holo_agent {
                            debug!("Joining code not created by holo_agent");
                            return Ok(ValidateCallbackResult::Invalid("Joining code invalid: incorrect holo agent".to_string()))
                        }
                        let signature = mem_proof.signature().clone();
                        if verify_signature(holo_agent.clone(), signature, mem_proof.header())? {
                            debug!("Joining code validated");
                            return Ok(ValidateCallbackResult::Valid)
                        } else {
                            debug!("Joining code validation failed: incorrect signature");
                            return Ok(ValidateCallbackResult::Invalid("Joining code invalid: incorrect signature".to_string()))
                        }
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
pub(crate) fn get_my_agent_validation_pkg() -> ExternResult<Element> {
    let filter = QueryFilter::new();
    let header_filter = filter.header_type(HeaderType::AgentValidationPkg);
    let query_result: Vec<Element> = query(header_filter)?;
    // There should be only one AgentValidationPkg per source chain
    Ok(query_result[0].clone())
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
                let element_pkg = match agent_info() {
                    Ok(ai) => {
                        if ai.agent_initial_pubkey == *data.element.header().author() {
                            debug!("Self Validating the AgentValidationPkg...");
                            Some(get_my_agent_validation_pkg()?)
                        } else {
                            return match get(header.clone(), GetOptions::default()) {
                                Ok(e) => e,
                                Err(_) => return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![(header.clone()).into()]))
                            }
                        }
                    }
                    _ => {
                        Some(get_my_agent_validation_pkg()?)
                    }
                };
                match element_pkg {
                   Some(agent_validation_pkg) => {
                       return joining_code(agent_validation_pkg)
                   },
                   None => return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![(header.clone()).into()]))
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
