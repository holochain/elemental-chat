use hdk::prelude::*;
use crate::message::Message;

// TODO: add checking of property
// This is useful for test cases where we don't want to provide a membrane proof
pub(crate) fn skip_proof() -> bool {
    return false
}

/// This is the current structure of the payload the holo signs
#[hdk_entry(id = "joining_code_payload")]
#[derive(Clone)]
pub(crate) struct JoiningCodePayload {
    pub role: String,
    pub record_locator: String
}

pub(crate) fn joining_code_value(mem_proof: &Element) -> String {
    //let code = mem_proof.entry().to_app_option::<validation::JoiningCodePayload>()?.unwrap();
    mem_proof.header_address().to_string()
}

/// check to see if this is the valid read_only membrane proof
pub(crate) fn is_read_only_proof(mem_proof: &MembraneProof) -> bool {
    if skip_proof() {
        return false;
    }
    let b = mem_proof.bytes();
    b == &[0]
}


/// Validate joining code from the membrane_proof
pub(crate) fn joining_code(author: AgentPubKey, membrane_proof: Option<MembraneProof>, genesis: bool) -> ExternResult<ValidateCallbackResult> {

    if skip_proof() {
        return Ok(ValidateCallbackResult::Valid);
    }

    // This is a hard coded holo agent public key
    let holo_agent = AgentPubKey::try_from("uhCAkRHEsXSAebzKJtPsLY1XcNePAFIieFBtz2ATanlokxnSC1Kkz").unwrap();
    match membrane_proof {
        Some(mem_proof) => {
            if is_read_only_proof(&mem_proof) {
                return Ok(ValidateCallbackResult::Valid)
            };
            let mem_proof = match Element::try_from(mem_proof.clone()) {
                Ok(m) => m,
                Err(e) => return Ok(ValidateCallbackResult::Invalid(format!("Joining code invalid: unable to deserialize into element ({:?})", e)))
            };

            trace!("Joining code provided: {:?}", mem_proof);

            let joining_code_author = mem_proof.header().author().clone();

            if joining_code_author != holo_agent {
                trace!("Joining code validation failed");
                return Ok(ValidateCallbackResult::Invalid(format!("Joining code invalid: unexpected author ({:?})", joining_code_author)))
            }

            let e = mem_proof.entry();
            if let ElementEntry::Present(_entry) = e {
                let signature = mem_proof.signature().clone();
                match verify_signature(holo_agent.clone(), signature, mem_proof.header()) {
                    Ok(verified) => {
                        if verified {
                            trace!("Joining code validated");
                            if !genesis {
                                let code = joining_code_value(&mem_proof);
                                trace!("Checking for joining code: {}", code);
                                let path = Path::from(code.clone());
                                let path_entry_hash = path.hash()?;
                                let maybe_details = get_details(path_entry_hash.clone(), GetOptions::default())?;
                                match maybe_details {
                                    Some(details) => {
                                        if let Details::Entry(e) = details {
                                            let mut deets:Vec<(Timestamp, AgentPubKey)> = e.headers.iter().map(|h| {
                                                let header = h.header();
                                                (header.timestamp(), header.author().clone())
                                            }).collect();
                                            deets.sort_by(|a, b| a.0.cmp(&b.0));
                                            if deets[0].1 != author.clone() {
                                                return Ok(ValidateCallbackResult::Invalid(format!("Earliest joining code for {} was by {} not {} as expected", code, deets[0].1, author )))
                                            }
                                        }
                                    }
                                    None => {
                                        trace!("Unresolved, waiting...");
                                        return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![(path_entry_hash).into()]))
                                    }
                                };
                            }
                            return Ok(ValidateCallbackResult::Valid)
                        } else {
                            trace!("Joining code validation failed: incorrect signature");
                            return Ok(ValidateCallbackResult::Invalid("Joining code invalid: incorrect signature".to_string()))
                        }
                    },
                    Err(e) => {
                        debug!("Error on get when verifying signature of agent entry: {:?}; treating as unresolved dependency",e);
                        return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![(author).into()]))
                    }
                }
            } else {
                return Ok(ValidateCallbackResult::Invalid("Joining code invalid payload".to_string()));
            }

        }
        None => Ok(ValidateCallbackResult::Invalid("No membrane proof found".to_string()))
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
                match get(header.clone(), GetOptions::default()) {
                    Ok(element_pkg) => match element_pkg {
                        Some(element_pkg) => {
                            match element_pkg.signed_header().header() {
                                Header::AgentValidationPkg(pkg) => {
                                    return joining_code(pkg.author.clone(), pkg.membrane_proof.clone(), false)
                                }
                                _ => return Ok(ValidateCallbackResult::Invalid("No Agent Validation Pkg found".to_string()))
                            }
                        },
                        None => return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![(header.clone()).into()]))
                    },
                    Err(e) => {
                        debug!("Error on get when validating agent entry: {:?}; treating as unresolved dependency",e);
                        return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![(header.clone()).into()]))
                    }
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
