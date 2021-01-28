use chat::channel::Channel;
use chat::message::Chunk;
use chat::ChannelData;
use chat::ChannelInput;
use chat::ChannelList;
use chat::ChannelListInput;
use chat::ListMessages;
use chat::ListMessagesInput;
use chat::Message;
use chat::MessageData;
use chat::MessageInput;
use chat::SigResults;
use chat::SignalMessageData;
use chat::SignalPayload;
use holochain::conductor::config::ConductorConfig;
use holochain::test_utils::consistency_10s;
use holochain::test_utils::consistency_10s_others;
// use holochain::test_utils::show_authored;
// use holochain::test_utils::show_authored_ops;
use holochain::test_utils::sweetest::*;
use holochain_types::signal::Signal;
use observability::tracing::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use stream_cancel::Valve;

#[tokio::test(threaded_scheduler)]
async fn behavior() {
    observability::test_run().ok();
    const NUM_CONDUCTORS: usize = 20;
    const NUM_MESSAGES: usize = 40;

    let dna_path = std::env::current_dir()
        .unwrap()
        .join("../../elemental-chat.dna.gz");
    let dna = SweetDnaFile::from_file(&dna_path).await.unwrap();
    let dna = dna.with_uuid(nanoid::nanoid!()).await.unwrap();

    let network = SweetNetwork::env_var_proxy().unwrap_or_else(|| {
        info!("KIT_PROXY not set using local quic network");
        SweetNetwork::local_quic()
    });
    let mut config = ConductorConfig::default();
    config.network = Some(network);

    let mut conductors = SweetConductorBatch::from_config(NUM_CONDUCTORS, config).await;
    let cells = conductors.setup_app("app", &[dna]).await;
    let cells = cells.cells_flattened();
    let zomes = cells
        .iter()
        .map(|cell| cell.zome("chat"))
        .collect::<Vec<_>>();
    conductors.exchange_peer_info().await;

    let channel = Channel {
        category: "category".into(),
        uuid: "uuid".into(),
    };
    let channel_data: ChannelData = conductors[0]
        .call(
            &zomes[0],
            "create_channel",
            ChannelInput {
                name: "name".into(),
                channel: channel.clone(),
            },
        )
        .await;
    // let envs = cells.iter().map(|c| c.env()).collect::<Vec<_>>();
    // show_authored(&envs);
    consistency_10s(&cells[..]).await;
    // show_authored_ops(&envs).await;
    let channels: ChannelList = conductors[0]
        .call(
            &zomes[0],
            "list_channels",
            ChannelListInput {
                category: "category".into(),
            },
        )
        .await;
    assert_eq!(channels.channels[0].channel, channel);
    let mut uuids = HashSet::new();
    let mut uuid_counts = HashMap::new();
    // Make the channel buffer big enough to not block
    let (resp, mut recv) = tokio::sync::mpsc::channel(NUM_CONDUCTORS * NUM_MESSAGES * 2);
    let mut jhs = Vec::new();
    let (trigger, valve) = Valve::new();
    let total_recv = Arc::new(AtomicUsize::new(0));
    for c in conductors.iter_mut() {
        use futures::stream::StreamExt;
        let mut stream = valve.wrap(c.signals().await);
        let jh = tokio::task::spawn({
            let mut resp = resp.clone();
            let total_recv = total_recv.clone();
            async move {
                while let Some(Signal::App(_, signal)) = stream.next().await {
                    let signal: SignalPayload = signal.into_inner().unwrap();
                    if let SignalPayload::Message(SignalMessageData {
                        message_data:
                            MessageData {
                                message: Message { uuid, .. },
                                ..
                            },
                        ..
                    }) = signal
                    {
                        total_recv.fetch_add(1, Ordering::Relaxed);
                        resp.send(uuid).await.expect("Failed to send uuid");
                    }
                }
            }
        });
        jhs.push(jh);
    }
    for (c, z) in conductors.iter().zip(zomes.iter()) {
        let _: () = c.call(z, "refresh_chatter", ()).await;
    }
    consistency_10s_others(&cells[..]).await;
    for (c, z) in conductors.iter().zip(zomes.iter()) {
        for _ in 0..NUM_MESSAGES {
            let uuid = nanoid::nanoid!();
            uuids.insert(uuid.clone());
            uuid_counts.insert(uuid.clone(), 0);
            let md: MessageData = c
                .call(
                    &z,
                    "create_message",
                    MessageInput {
                        last_seen: chat::message::LastSeen::First,
                        channel: channel.clone(),
                        message: Message {
                            uuid,
                            content: "Hey".to_string(),
                        },
                        chunk: 0,
                    },
                )
                .await;
            let r: SigResults = c
                .call(
                    &z,
                    "signal_chatters",
                    SignalMessageData {
                        message_data: md,
                        channel_data: channel_data.clone(),
                    },
                )
                .await;
            debug!(total_sent = ?r.total);
        }
    }
    let mut done = 0;
    while let Ok(Some(uuid)) =
        tokio::time::timeout(std::time::Duration::from_secs(5), recv.recv()).await
    {
        let count = uuid_counts
            .get_mut(&uuid)
            .expect("Found uuid that doesn't exist");
        *count += 1;
        if *count >= NUM_CONDUCTORS - 1 {
            done += 1;
        }
        if done == NUM_MESSAGES * NUM_CONDUCTORS {
            debug!("Done");
            std::mem::drop(recv);
            break;
        } else {
            let counts = uuid_counts.values().collect::<Vec<_>>();
            warn!(?done, ?counts);
        }
    }
    let total_recv = total_recv.load(Ordering::Relaxed);
    debug!(?total_recv);

    consistency_10s_others(&cells[..]).await;
    for (c, z) in conductors.iter().zip(zomes.iter()) {
        let messages: ListMessages = c
            .call(
                z,
                "list_messages",
                ListMessagesInput {
                    channel: channel.clone(),
                    chunk: Chunk { start: 0, end: 1 },
                    active_chatter: true,
                },
            )
            .await;
        assert!(uuids.contains(&messages.messages[0].message.uuid));
    }
    trigger.cancel();
    for jh in jhs {
        jh.await.unwrap();
    }
}
