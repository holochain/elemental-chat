use holochain_deterministic_integrity::prelude::*;

pub struct GenericLink;
impl GenericLink {
    const TAG: &'static [u8; 7] = b"generic";
    /// Create the tag
    pub fn tag() -> LinkTag {
        LinkTag::new(*Self::TAG)
    }
}

#[hdk_link_types]
pub enum LinkTypes {
    GenericLink,
}

#[hdk_entry_helper]
pub struct JoiningCode(pub String);

#[hdk_entry_helper]
pub struct TestObj {
    pub value: String,
}

#[hdk_entry_defs]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    TestObj(TestObj),
}
