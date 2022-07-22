// use chat::channel::*;
// use chat::message::*;
// use chat::*;
// use hc_joining_code::Props;
// use holochain::conductor::api::error::{ConductorApiError, ConductorApiResult};
// use holochain::sweettest::*;

// #[cfg(test)]
// #[tokio::test(flavor = "multi_thread")]
// async fn chat_away() {
//     // Use prebuilt DNA bundle.
//     // You must build the DNA bundle as a separate step before running the test.
//     let dna_path = std::env::current_dir()
//         .unwrap()
//         .join("../../elemental-chat.dna");

//     // Convert to DnaFile and apply property overrides
//     let dna = SweetDnaFile::from_bundle_with_overrides(
//         &dna_path,
//         None,
//         // Note that we can use our own native `Props` type
//         Some(Props {
//             skip_proof: true,
//             holo_agent_override: None,
//             development_stage: None,
//             t_and_c: None,
//             t_and_c_agreement: None,
//         }),
//     )
//     .await
//     .unwrap();

//     // Set up conductor
//     let mut conductor = SweetConductor::from_standard_config().await;

//     let agents = SweetAgents::get(conductor.keystore(), 2).await;

//     // Install apps with single DNA
//     let apps = conductor
//         .setup_app_for_agents("elemental-chat", &agents, &[dna])
//         .await
//         .unwrap();
//     let ((alice_cell,), (bobbo_cell,)) = apps.into_tuples();
//     let alice_chat = &alice_cell.zome("chat");
//     let bobbo_chat = &bobbo_cell.zome("chat");

//     // Setup complete.

//     let channel: ChannelData = conductor
//         .call(
//             alice_chat,
//             "create_channel",
//             ChannelInput {
//                 name: "Test Ch".into(),
//                 entry: Channel {
//                     category: "General".into(),
//                     uuid: uuid::Uuid::new_v4().to_string(),
//                 },
//             },
//         )
//         .await;

//     let long_msg = MessageInput {
//         last_seen: LastSeen::First,
//         channel: channel.entry.clone(),
//         entry: Message {
//             uuid: "long msg".into(),
//             content: std::iter::repeat('x').take(1025).collect(),
//         },
//     };

//     let error: ConductorApiResult<MessageData> = conductor
//         .call_fallible(alice_chat, "create_message", long_msg.clone())
//         .await;

//     assert!(matches!(error, Err(ConductorApiError::CellError(_))));

//     let mut msg0 = long_msg;
//     msg0.entry.uuid = "msg0".into();
//     msg0.entry.content = "Hello from alice :)".into();

//     let res0: MessageData = conductor
//         .call(alice_chat, "create_message", msg0.clone())
//         .await;

//     assert_eq!(msg0.entry, res0.entry);

//     let mut msg1 = msg0.clone();
//     msg1.last_seen = LastSeen::Message(res0.entry_hash.clone());
//     msg1.entry.uuid = "msg1".into();
//     msg1.entry.content = "Is anybody out there?".into();

//     let res1: MessageData = conductor
//         .call(alice_chat, "create_message", msg1.clone())
//         .await;

//     assert_eq!(msg1.entry, res1.entry);

//     // let current_time = Utc::now();
//     let lmpi = ListMessagesInput {
//         channel: channel.entry.clone(),
//         earliest_seen: None,
//         target_message_count: 1,
//     };

//     let alice_msgs: ListMessages = conductor
//         .call(alice_chat, "list_messages", lmpi.clone())
//         .await;
//     println!(">{:?}", alice_msgs);
//     // TODO: add consistency awaiting to sweettest
//     tokio::time::sleep(tokio::time::Duration::from_millis(4000)).await;

//     let bobbo_msgs: ListMessages = conductor
//         .call(bobbo_chat, "list_messages", lmpi.clone())
//         .await;

//     // Alice got all messages so far.
//     assert_eq!(
//         alice_msgs.messages.as_slice(),
//         &[res0.clone(), res1.clone()]
//     );
//     // Bobbo got the same messages as Alice.
//     assert_eq!(alice_msgs, bobbo_msgs);

//     let mut msg2 = msg1.clone();
//     msg2.last_seen = LastSeen::Message(res1.entry_hash.clone());
//     msg2.entry.uuid = "msg2".into();
//     msg2.entry.content = "I'm here".into();

//     let mut msg3 = msg1.clone();
//     msg3.last_seen = LastSeen::Message(res1.entry_hash.clone());
//     msg3.entry.uuid = "msg3".into();
//     msg3.entry.content = "Anybody?".into();

//     let res2: MessageData = conductor
//         .call(alice_chat, "create_message", msg2.clone())
//         .await;
//     let res3: MessageData = conductor
//         .call(bobbo_chat, "create_message", msg3.clone())
//         .await;

//     // TODO: add consistency awaiting to sweettest
//     tokio::time::sleep(tokio::time::Duration::from_millis(4000)).await;

//     let alice_msgs: ListMessages = conductor
//         .call(alice_chat, "list_messages", lmpi.clone())
//         .await;

//     let bobbo_msgs: ListMessages = conductor
//         .call(bobbo_chat, "list_messages", lmpi.clone())
//         .await;

//     // Alice got all messages so far.
//     assert_eq!(
//         alice_msgs.messages.as_slice(),
//         &[res0.clone(), res1.clone(), res2.clone(), res3.clone()]
//     );
//     // Bobbo got the same messages as Alice.
//     println!("{:?}", alice_msgs);
//     assert_eq!(alice_msgs, bobbo_msgs);
// }
