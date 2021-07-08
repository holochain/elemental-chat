use hdk::prelude::*;

use crate::joining_code;

pub fn init_validate_and_create_joining_code() -> ExternResult<InitCallbackResult> {
    let elements = &query(ChainQueryFilter::new().header_type(HeaderType::AgentValidationPkg))?;
    if let Header::AgentValidationPkg(h) = elements[0].header() {
        match &h.membrane_proof {
            Some(mem_proof) => {
                let mem_proof = match Element::try_from(mem_proof.clone()) {
                    Ok(m) => m,
                    Err(_e) => {
                        return Ok(InitCallbackResult::Fail(
                            "Malformed membrane proof: it is not an element".into(),
                        ))
                    }
                };
                let code = joining_code::joining_code_value(&mem_proof);

                trace!("looking for {}", code);
                let path = Path::from(code.clone());
                if path.exists()? {
                    return Ok(InitCallbackResult::Fail(format!(
                        "membrane proof for {} already used",
                        code
                    )));
                }
                trace!("creating {:?}", code);
                path.ensure()?;

                Ok(InitCallbackResult::Pass)
            }
            None => {
                return Ok(InitCallbackResult::Fail(
                    "There is no membrane proof".into(),
                ))
            }
        }
    } else {
        return Ok(InitCallbackResult::Fail("No elements in the chain".into()));
    }
}
