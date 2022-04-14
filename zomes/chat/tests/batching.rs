#![allow(dead_code)]
// use chat::channel::*;
// use chat::message::*;
use chat::*;
use chrono::TimeZone;
// use hc_joining_code::Props;
// use holochain::conductor::api::error::{ConductorApiError, ConductorApiResult};
// use holochain::sweettest::*;
use proptest::prelude::*;

// Two main time consuming parts to this test:
// - Writing the zome call
// - Getting the holochain setup right. (What part of the holochain setup can we re-use between iterations?)
// - Using proptest for my first time and figuring out how to use it
//
// Possible next steps:
// - [x] Get to a place where we have the proptest randomizing, and can print the inputs
// - [ ] Get to a place where we have the proptest randomizing and holochain usable in each iteration
// - [ ] Write the zome call

#[derive(Debug)]
struct InsertFakeMessagePayload {
    content: String,
    timestamp: Timestamp,
}

prop_compose! {
    fn generate_timestamp()(
        hour in (0_u32..3),
        day in (0_u32..3),
        month in (0_u32..3),
        year in (0_i32..3)
    ) -> Timestamp {
        Timestamp::from(chrono::Utc.ymd(2022 + year, 1 + month, 1 + day).and_hms(hour, 0, 0))
    }
}

prop_compose! {
    fn generate_message_history(length: usize)(
        timestamps in prop::collection::vec(generate_timestamp(), length)
    ) -> Vec<InsertFakeMessagePayload> {
        timestamps.into_iter().enumerate().map(|(i, timestamp)| InsertFakeMessagePayload { content: format!("{}", i), timestamp}).collect()
    }
}

#[derive(Debug)]
struct TestInput {
    length: usize,
    message_history: Vec<InsertFakeMessagePayload>,
    index: usize,
}

prop_compose! {
    fn generate_test_input(length: usize)(
        message_history in generate_message_history(length),
        index in (0..length)
    ) -> TestInput {
        TestInput {
            length,
            message_history,
            index,
        }
    }
}

proptest! {
    #[test]
    fn test_batching(test_input in generate_test_input(90)) {
        dbg!(test_input.length, test_input.message_history, test_input.index);
        // for message in message_history {
        //     // Insert a message into the DHT at an arbitrary timestamp
        //     zome_call("insert_fake_message", message)
        // }

        // let earliest_seen = message_history[index];
        // let messages = timeout(zome_call("list_messages", earliest_seen), ...);

        // assert!(messages.all(|m| m.timestamp < earliest_seen.timestamp));
        // assert!(messages.is_sorted());
    }
}

// proptest! {
//     #![proptest_config(ProptestConfig::with_cases(10))]
//     #[test]
//     fn test_batching(length in 1000i32..1001) {
//         match tokio::runtime::Runtime::new() {
//             Ok(rt) => {
//                 let _ = rt.block_on(async move {

//                     // Use prebuilt DNA bundle.
//                     // You must build the DNA bundle as a separate step before running the test.
//                     let dna_path = std::env::current_dir()
//                         .unwrap()
//                         .join("../../elemental-chat.dna");

//                     // Convert to DnaFile and apply property overrides
//                     let dna = SweetDnaFile::from_bundle_with_overrides(
//                         &dna_path,
//                         None,
//                         // Note that we can use our own native `Props` type
//                         Some(Props {
//                             skip_proof: true,
//                             holo_agent_override: None,
//                             development_stage: None,
//                             t_and_c: None,
//                             t_and_c_agreement: None,
//                         }),
//                     )
//                     .await
//                     .unwrap();

//                     // Set up conductor
//                     let mut conductor = SweetConductor::from_standard_config().await;

//                     let agents = SweetAgents::get(conductor.keystore(), 2).await;

//                     // Install apps with single DNA
//                     let apps = conductor
//                         .setup_app_for_agents("elemental-chat", &agents, &[dna])
//                         .await
//                         .unwrap();
//                     let ((alice_cell,), (_bobbo_cell,)) = apps.into_tuples();
//                     let alice_chat = &alice_cell.zome("chat");
//                     // let bobbo_chat = &bobbo_cell.zome("chat");

//                     // Setup complete.

//                     let channel: ChannelData = conductor
//                     .call(
//                         alice_chat,
//                         "create_channel",
//                         ChannelInput {
//                             name: "Test Ch".into(),
//                             entry: Channel {
//                                 category: "General".into(),
//                                 uuid: uuid::Uuid::new_v4().to_string(),
//                             },
//                         },
//                     )
//                     .await;

//                     let _: () = conductor.call(alice_chat, "create_test_messages", message::handlers::InsertTestMessages {
//                         channel: channel.entry.clone(),
//                         number_of_messages: length.clone(),
//                     }).await;
//                     let time = Timestamp::now();
//                     let lmpi = ListMessagesInput {
//                         channel: channel.entry.clone(),
//                         earliest_seen: Some(time),
//                         target_message_count: length as usize + 2,
//                     };

//                     let alice_msgs: ListMessages = conductor
//                     .call(alice_chat, "list_messages", lmpi.clone())
//                     .await;

//                     assert_eq!(
//                         alice_msgs.messages.len(),
//                         length as usize + 2
//                     );

//                     assert!(alice_msgs.messages.iter().all(|m| m.created_at < time));

//                     conductor.shutdown().await;
//                 });
//             }
//             _ => println!("Cannot spin up async test")
//         }

//     }
// }
