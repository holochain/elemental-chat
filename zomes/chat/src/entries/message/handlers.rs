use crate::message::{
    MessageEntry,
    MessageInput,
    Message,
};

#[hdk_extern]
fn create_message(message_input: MessageInput) -> ExternResult<Message> {
    let header_hash = commit_entry!(&message_input.message_entry)?;
    let element = get!(&header_hash)??;
    let entry_hash = element.header().entry_hash()?;
    let message = Message::try_from(element)?
    link_entries!(message_input.base_entry_hash, entry_hash)?;
    Ok(message)
}
