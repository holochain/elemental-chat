mod entries;

use element::{Element, ElementEntry};
use entries::{Channel, ChannelList, ChannelMessage, ChannelMessageList, ChannelName};
use hdk3::prelude::Path;
use hdk3::prelude::*;
use link::Link;

holochain_externs!();

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
    commit_entry!(&channel)?;
    let channel_hash = entry_hash!(&channel)?;
    debug!(format!(
        "channel hash {:?}",
        SerializedBytes::try_from(channel_hash.clone())
    ))?;
    link_entries!(entry_hash!(&path)?, channel_hash.clone())?;
    Ok(channel_hash)
}

fn _create_message(input: CreateMessageInput) -> WasmResult<()> {
    let CreateMessageInput {
        channel_hash,
        content,
    } = input;
    let message = ChannelMessage::new(content);
    commit_entry!(&message)?;
    link_entries!(channel_hash, entry_hash!(&message)?)?;
    Ok(())
}

fn _list_channels(_: ()) -> WasmResult<ChannelList> {
    let path_hash = entry_hash!(channels_path())?;
    let links: Vec<Link> = get_links!(path_hash)?.into();
    let channels: Vec<Channel> = links
        .into_iter()
        .map(|link| get_entry!(link.target))
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
        .map(|link| get_entry!(link.target))
        .flatten()
        .flatten()
        .map(|el| entry_from_element(el).and_then(|sb| Ok(ChannelMessage::try_from(sb)?)))
        .collect::<WasmResult<_>>()?;
    Ok(messages.into())
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
struct CreateMessageInput {
    channel_hash: EntryHash,
    content: String,
}

map_extern!(create_channel, _create_channel);
map_extern!(create_message, _create_message);
map_extern!(list_channels, _list_channels);
map_extern!(list_messages, _list_messages);

#[cfg(test)]
mod tests {
    use super::*;
    use holo_hash::{hash_type, HoloHash};

    #[test]
    fn ser() {
        let hash = HoloHash::from_raw_bytes_and_type(
            vec![
                118, 162, 157, 84, 135, 189, 154, 240, 188, 86, 55, 53, 222, 211, 181, 149, 254,
                34, 251, 198, 246, 121, 223, 51, 212, 160, 205, 73, 110, 31, 188, 67, 135, 117,
                249, 97,
            ],
            hash_type::Content,
        )
        .into();
        let x = dbg!(SerializedBytes::try_from(CreateMessageInput {
            channel_hash: hash,
            content: "Hello from alice :)".into()
        })
        .unwrap());
        println!("{:?}", x.bytes());
        let x = dbg!(CreateMessageInput::try_from(x).unwrap());
    }
}
