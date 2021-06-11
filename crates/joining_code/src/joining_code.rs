use hdk::prelude::*;

/// This is the current structure of the payload the holo signs
#[hdk_entry(id = "joining_code_payload")]
#[derive(Clone)]
pub struct JoiningCodePayload {
    pub role: String,
    pub record_locator: String,
}

pub fn joining_code_value(mem_proof: &Element) -> String {
    //let code = mem_proof.entry().to_app_option::<validation::JoiningCodePayload>()?.unwrap();
    mem_proof.header_address().to_string()
}
