use crate::message::Message;
use hdk::prelude::*;

pub(crate) fn common_validatation(data: ValidateData) -> ExternResult<ValidateCallbackResult> {
    let element = data.element.clone();
    let entry = element.into_inner().1;
    let entry = match entry {
        ElementEntry::Present(e) => e,
        _ => return Ok(ValidateCallbackResult::Valid),
    };
    if let Entry::Agent(_) = entry {
        if !hc_joining_code::skip_proof() {
            match data.element.header().prev_header() {
                Some(header) => {
                    let signed_header_hashed: SignedHeaderHashed = must_get_header(header.clone())?;
                    let header: Header = signed_header_hashed.into();
                    match header {
                        Header::AgentValidationPkg(pkg) => {
                            return hc_joining_code::validate_joining_code(
                                hc_joining_code::holo_agent(&zome_info()?.properties)?,
                                pkg.author.clone(),
                                pkg.membrane_proof.clone(),
                            )
                        }
                        _ => {
                            return Ok(ValidateCallbackResult::Invalid(
                                "No Agent Validation Pkg found".to_string(),
                            ))
                        }
                    };
                }
                None => unreachable!("This element will always have a prev_header"),
            }
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
