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
async fn signal_behavior() {
    observability::test_run().ok();
    const NUM_CONDUCTORS: usize = 15;
    const NUM_MESSAGES: usize = 40;
    let (num_conductors, num_messages) = std::env::var_os("HC_NUM")
        .and_then(|input| {
            let input = input.into_string().unwrap();
            let mut input = input.split(',');
            let s = input.next()?;
            let num_conductors = str::parse::<usize>(s).unwrap();
            let s = input.next()?;
            let num_messages = str::parse::<usize>(s).unwrap();
            Some((num_conductors, num_messages))
        })
        .unwrap_or((NUM_CONDUCTORS, NUM_MESSAGES));

    let dna_path = std::env::current_dir()
        .unwrap()
        .join("../../elemental-chat.dna.gz");
    let dna = SweetDnaFile::from_file(&dna_path).await.unwrap();
    let dna = dna.with_uuid(nanoid::nanoid!()).await.unwrap();

    let mut network = SweetNetwork::env_var_proxy().unwrap_or_else(|| {
        info!("KIT_PROXY not set using local quic network");
        SweetNetwork::local_quic()
    });

    // Set remote call to 10s and therefor remote signal
    network.tuning_params.default_rpc_single_timeout_ms = 20000;

    let mut config = ConductorConfig::default();
    config.network = Some(network);

    let mut conductors = SweetConductorBatch::from_config(num_conductors, config).await;
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
    let (resp, mut recv) = tokio::sync::mpsc::channel(num_conductors * num_messages * 2);
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
                    let signal: SignalPayload = signal.into_inner().decode().unwrap();
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
        let start = std::time::Instant::now();
        for _ in 0..num_messages {
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
            let _r: SigResults = c
                .call(
                    &z,
                    "signal_chatters",
                    SignalMessageData {
                        message_data: md,
                        channel_data: channel_data.clone(),
                    },
                )
                .await;
        }
        let el = start.elapsed();
        let delay = num_messages * 300;
        let delay = std::time::Duration::from_millis(delay as u64);
        if let Some(wait) = el.checked_sub(delay) {
            debug!(waiting_for = %wait.as_millis());
            tokio::time::delay_for(wait).await;
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
        if *count >= num_conductors - 1 {
            done += 1;
        }
        if done == num_messages * num_conductors {
            debug!("Done");
            std::mem::drop(recv);
            break;
        } else {
            let counts = uuid_counts.values().collect::<Vec<_>>();
            warn!(?done, ?counts);
        }
    }
    assert_eq!(done, num_messages * num_conductors);
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

#[tokio::test(threaded_scheduler)]
async fn gossip_behavior() {
    observability::test_run_dead().ok();
    tokio::spawn(async {
        loop {
            observability::tick_deadlock_catcher();
            tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
        }
    });

    const NUM_CONDUCTORS: usize = 15;
    const NUM_MESSAGES: usize = 40;
    const SEND_RATE_MS: usize = 300;
    let (num_conductors, num_messages, send_rate) = std::env::var_os("HC_NUM")
        .and_then(|input| {
            let input = input.into_string().unwrap();
            let mut input = input.split(',');
            let s = input.next()?;
            let num_conductors = str::parse::<usize>(s).unwrap();
            let s = input.next()?;
            let num_messages = str::parse::<usize>(s).unwrap();
            let s = input.next()?;
            let send_rate = str::parse::<usize>(s).unwrap();
            Some((num_conductors, num_messages, send_rate))
        })
        .unwrap_or((NUM_CONDUCTORS, NUM_MESSAGES, SEND_RATE_MS));

    let dna_path = std::env::current_dir()
        .unwrap()
        .join("../../elemental-chat.dna.gz");
    let dna = SweetDnaFile::from_file(&dna_path).await.unwrap();
    let dna = dna.with_uuid(nanoid::nanoid!()).await.unwrap();

    let mut network = SweetNetwork::env_var_proxy().unwrap_or_else(|| {
        info!("KIT_PROXY not set using local quic network");
        SweetNetwork::local_quic()
    });

    // network.tuning_params.default_notify_remote_agent_count = num_conductors as u32;
    network.tuning_params.default_notify_remote_agent_count = 5;

    let mut config = ConductorConfig::default();
    config.network = Some(network);

    let mut conductors = SweetConductorBatch::from_config(num_conductors, config).await;
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
    let _: ChannelData = conductors[0]
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

    let mut jhs = Vec::with_capacity(num_conductors);
    for ((c_num, c), z) in conductors
        .into_inner()
        .into_iter()
        .enumerate()
        .zip(zomes.into_iter())
    {
        let channel = channel.clone();
        let jh = tokio::task::spawn(async move {
            let mut uuids = HashSet::new();
            let start = std::time::Instant::now();
            for i in 0..num_messages {
                let uuid = nanoid::nanoid!();
                uuids.insert(uuid.clone());
                let _: MessageData = c
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
                let num_sent = i + 1;
                let el = start.elapsed();
                let target = num_sent * send_rate;

                let target = std::time::Duration::from_millis(target as u64);
                if let Some(wait) = target.checked_sub(el) {
                    debug!(waiting_for = %wait.as_millis());
                    tokio::time::delay_for(wait).await;
                }
            }
            (c_num, c, z, uuids)
        });
        jhs.push(jh);
    }

    let mut conductors = Vec::with_capacity(num_conductors);
    let mut zomes = Vec::with_capacity(num_conductors);
    let mut uuids = HashSet::new();
    for jh in jhs {
        let (c_num, conductor, zome, u) = jh.await.unwrap();
        conductors.push((c_num, conductor));
        zomes.push((c_num, zome));
        uuids.extend(u);
    }

    let start = std::time::Instant::now();

    consistency_10s_others(&cells[..]).await;

    debug!(consistency_in_s = %start.elapsed().as_secs());
    // Put the conductors and zomes into the same order
    conductors.sort_by_key(|(k, _)| *k);
    let conductors: Vec<_> = conductors.into_iter().map(|(_, c)| c).collect();
    zomes.sort_by_key(|(k, _)| *k);
    let zomes: Vec<_> = zomes.into_iter().map(|(_, c)| c).collect();

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
        let messages: HashSet<_> = messages
            .messages
            .into_iter()
            .map(|m| m.message.uuid)
            .collect();
        let num_missing = uuids.difference(&messages).count();
        assert_eq!(num_missing, 0);
    }
}
