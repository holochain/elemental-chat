use holochain_deterministic_integrity::prelude::*;

/// The actual message data that is saved into the DHT
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Message {
    pub uuid: String,
    pub content: String,
}

/// The actual channel data that is saved into the DHT
/// This is the actual name of the channel that
/// can change.
#[hdk_entry_helper]
#[derive(Clone, PartialEq, Eq)]
pub struct ChannelInfo {
    pub category: String,
    pub uuid: String,
    pub name: String,
    pub created_by: AgentPubKey,
    pub created_at: Timestamp,
}

#[hdk_entry_defs]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_def(visibility = "public", required_validations = 2)]
    Message(Message),
    #[entry_def(visibility = "public", required_validations = 2)]
    ChannelInfo(ChannelInfo),
}

#[hdk_link_types]
pub enum LinkTypes {
    Channel,
    Chatter,
    Message,
}
