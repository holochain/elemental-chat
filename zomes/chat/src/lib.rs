mod entries;

use element::{Element, ElementEntry};
use entries::{Channel, ChannelList, ChannelMessage, ChannelMessageList, ChannelName};
use hdk3::prelude::Path;
use hdk3::prelude::*;
use link::Link;

holochain_wasmer_guest::holochain_externs!();

type WasmResult<T> = Result<T, WasmError>;

fn error<T>(reason: &str) -> WasmResult<T> {
    Err(WasmError::Zome(reason.into()))
}

fn entry_from_element(element: Element) -> WasmResult<SerializedBytes> {
    match element.entry() {
        ElementEntry::Present(Entry::App(sb)) => Ok(sb.to_owned()),
        _ => error("Unexpected non-public or non-app entry. (No info to show.)"),
    }
}

entry_defs!(vec![
    Path::entry_def(),
    Channel::entry_def(),
    ChannelMessage::entry_def()
]);

fn channels_path() -> Path {
    let path = Path::from("channels");
    path.ensure().expect("Couldn't ensure path");
    path
}

fn _create_channel(name: ChannelName) -> WasmResult<EntryHash> {
    debug!(format!("channel name {:?}", name))?;
    let path = channels_path();
    let channel = Channel::new(name.into());
    let channel_hash = entry_hash!(&channel)?;
    commit_entry!(&channel)?;
    link_entries!(entry_hash!(&path)?, channel_hash.clone())?;
    Ok(channel_hash)
}

fn _create_message(input: CreateMessageInput) -> WasmResult<EntryHash> {
    let CreateMessageInput {
        channel_hash,
        content,
    } = input;
    let message = ChannelMessage::new(content);
    let message_hash = entry_hash!(&message)?;
    commit_entry!(&message)?;
    link_entries!(channel_hash, message_hash.clone())?;
    Ok(message_hash)
}

fn _list_channels(_: ()) -> WasmResult<ChannelList> {
    let path_hash = entry_hash!(channels_path())?;
    let links: Vec<Link> = get_links!(path_hash)?.into();
    let channels: Vec<Channel> = links
        .into_iter()
        .map(|link| get!(link.target))
        .flatten()
        .flatten()
        .map(|el| entry_from_element(el).and_then(|sb| Ok(Channel::try_from(sb)?)))
        .collect::<WasmResult<_>>()?;
    Ok(channels.into())
}

fn _list_messages(channel_hash: EntryHash) -> WasmResult<ChannelMessageList> {
    let links: Vec<Link> = get_links!(channel_hash)?.into();
    let messages: Vec<ChannelMessage> = links
        .into_iter()
        .map(|link| get!(link.target))
        .flatten()
        .flatten()
        .map(|el| entry_from_element(el).and_then(|sb| Ok(ChannelMessage::try_from(sb)?)))
        .collect::<WasmResult<_>>()?;
    Ok(messages.into())
}

#[derive(Serialize, Deserialize, SerializedBytes)]
struct CreateMessageInput {
    channel_hash: EntryHash,
    content: String,
}

map_extern!(create_channel, _create_channel);
map_extern!(create_message, _create_message);
map_extern!(list_channels, _list_channels);
map_extern!(list_messages, _list_messages);
