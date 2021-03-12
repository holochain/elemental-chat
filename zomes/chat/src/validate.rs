use hdk3::prelude::*;
use crate::error::{ChatResult, ChatError};
use crate::message::Message;


#[derive(Serialize, Deserialize, SerializedBytes, Debug )]
pub struct JoiningCode(Signature);

/// Validate joining code from the membrane_proof
pub(crate) fn joining_code() -> ChatResult<()> {
    let filter = QueryFilter::new();
    let filter = filter.sequence_range(1..2);
    let query_result: ElementVec = query(filter)?;
    let holo_agent = AgentPubKey::try_from("uhCAk7wGO_N3Lm9-OU7mDhyLSTI4WBxHQ9pq98TQbwaqmxJkqbmFW").unwrap();

    match query_result.0[0].signed_header().header() {
        Header::AgentValidationPkg(pkg) => {
            match &pkg.membrane_proof {
                Some(mem_proof) => {
                    let joining_code: JoiningCode = JoiningCode::try_from(mem_proof.clone())?;
                    debug!("Joining code provided: {:?}", joining_code);

                    if verify_signature(holo_agent.clone(), joining_code.0, SerializedBytes::try_from(holo_agent)?)? {
                        debug!("Joining code valadated");
                        Ok(())
                    } else {
                        debug!("Joining code validation failed");
                        Err(ChatError::InitFailure)
                    }
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
