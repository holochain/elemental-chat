use crate::channel::{
    ChannelEntry,
    ChannelInput,
    Channel,
};

#[hdk_extern]
fn create_channel(channel_input: ChannelInput) -> ExternResult<Channel> {
    let path = Path::from(channel_input.path);
    path.ensure();
    let header_hash = commit_entry!(&channel_input.channel_entry)?;
    let element = get!(&header_hash)??;
    let entry_hash = element.header().entry_hash()?;
    let channel = Channel::try_from(element)?
    link_entries!(path.hash(), entry_hash)?;
    Ok(channel)
}
