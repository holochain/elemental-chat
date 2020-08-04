mod entries;

use entries::{Channel, ChannelList, ChannelMessage, ChannelMessageList, ChannelName};
use hdk3::prelude::Path;
use hdk3::prelude::*;
use link::Link;

holochain_wasmer_guest::holochain_externs!();

type WasmResult<T> = Result<T, WasmError>;

entry_defs!(vec![
    Path::entry_def(),
    Channel::entry_def(),
    ChannelMessage::entry_def()
]);

fn channels_path() -> Path {
    Path::from("channels")
}

fn _create_channel(name: ChannelName) -> WasmResult<()> {
    let path = channels_path();
    path.ensure()?;
    let channel = Channel::new(name.into());
    commit_entry!(&channel)?;
    link_entries!(entry_hash!(&path)?, entry_hash!(&channel)?)?;
    Ok(())
}

fn _create_message(input: CreateMessageInput) -> WasmResult<()> {
    let CreateMessageInput {
        channel_hash,
        content,
    } = input;
    let message = ChannelMessage::new(content);
    let message_hash = entry_hash!(&message)?;
    link_entries!(channel_hash, message_hash)?;
    Ok(())
}

fn _list_channels(_: ()) -> WasmResult<ChannelList> {
    let path_hash = entry_hash!(channels_path())?;
    let links: Vec<Link> = get_links!(path_hash)?.into();
    let channels: Vec<_> = links
        .into_iter()
        .map(|link| get_entry!(link.target))
        .flatten()
        .flatten()
        .collect();
    Ok(channels.into())
}

fn _list_messages(channel_hash: EntryHash) -> WasmResult<ChannelMessageList> {
    todo!("implement")
}

#[derive(Serialize, Deserialize, SerializedBytes)]
struct CreateMessageInput {
    channel_hash: EntryHash,
    content: String,
}

map_extern!(create_channel, _create_channel);
map_extern!(create_message, _create_message);
map_extern!(list_channels, _list_channels);
