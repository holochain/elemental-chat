use hdk::prelude::*;

/// check to see if this is the valid read_only membrane proof
pub(crate) fn is_read_only_proof(mem_proof: &MembraneProof) -> bool {
    let b = mem_proof.bytes();
    b == &[0]
}

/// Validate joining code from the membrane_proof
pub fn validate_joining_code(
    progenitor_agent: AgentPubKey,
    author: AgentPubKey,
    membrane_proof: Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    match membrane_proof {
        Some(mem_proof) => {
            // TODO: remove hack once we can pass unique codes for hosts
            if is_read_only_proof(&mem_proof) {
                return Ok(ValidateCallbackResult::Valid);
            };
            let mem_proof = match Element::try_from(mem_proof.clone()) {
                Ok(m) => m,
                Err(e) => {
                    return Ok(ValidateCallbackResult::Invalid(format!(
                        "Joining code invalid: unable to deserialize into element ({:?})",
                        e
                    )))
                }
            };

            trace!("Joining code provided: {:?}", mem_proof);

            let joining_code_author = mem_proof.header().author().clone();

            if joining_code_author != progenitor_agent {
                trace!("Joining code validation failed");
                return Ok(ValidateCallbackResult::Invalid(format!(
                    "Joining code invalid: unexpected author ({:?})",
                    joining_code_author
                )));
            }

            let e = mem_proof.entry();
            if let ElementEntry::Present(_entry) = e {
                let signature = mem_proof.signature().clone();
                match verify_signature(progenitor_agent.clone(), signature, mem_proof.header()) {
                    Ok(verified) => {
                        if verified {
                            // TODO: check that the joining code has the correct author key in it
                            // once this is added to the registration flow, e.g.:
                            // if mem_proof.payload().agent != author {
                            //    return Ok(ValidateCallbackResult::Invalid("Joining code invalid: incorrect agent key".to_string()))
                            // }
                            trace!("Joining code validated");
                            return Ok(ValidateCallbackResult::Valid);
                        } else {
                            trace!("Joining code validation failed: incorrect signature");
                            return Ok(ValidateCallbackResult::Invalid(
                                "Joining code invalid: incorrect signature".to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        debug!("Error on get when verifying signature of agent entry: {:?}; treating as unresolved dependency",e);
                        return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![
                            (author).into(),
                        ]));
                    }
                }
            } else {
                return Ok(ValidateCallbackResult::Invalid(
                    "Joining code invalid payload".to_string(),
                ));
            }
        }
        None => Ok(ValidateCallbackResult::Invalid(
            "No membrane proof found".to_string(),
        )),
    }
}
