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
                Some(header) => match get(header.clone(), GetOptions::default()) {
                    Ok(element_pkg) => match element_pkg {
                        Some(element_pkg) => match element_pkg.signed_header().header() {
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
                        },
                        None => {
                            return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![
                                (header.clone()).into(),
                            ]))
                        }
                    },
                    Err(e) => {
                        debug!("Error on get when validating agent entry: {:?}; treating as unresolved dependency",e);
                        return Ok(ValidateCallbackResult::UnresolvedDependencies(vec![
                            (header.clone()).into(),
                        ]));
                    }
                },
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

pub fn set_read_only_cap_tokens() -> ExternResult<()> {
    let mut functions: GrantedFunctions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "get_active_chatters".into()));
    functions.insert((zome_info()?.zome_name, "list_channels".into()));
    functions.insert((zome_info()?.zome_name, "list_messages".into()));
    functions.insert((zome_info()?.zome_name, "list_all_messages".into()));
    functions.insert((zome_info()?.zome_name, "stats".into()));
    functions.insert((zome_info()?.zome_name, "agent_stats".into()));
    functions.insert((zome_info()?.zome_name, "refresh_chatter".into()));
    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        access: ().into(),
        functions,
    })?;
    Ok(())
}
