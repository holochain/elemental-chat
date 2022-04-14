#![allow(dead_code)] // FIXME(timo): remove before merging

use std::sync::{
    atomic::{AtomicU32, Ordering::SeqCst},
    Arc,
};

// use chat::channel::*;
// use chat::message::*;
use chat::*;
use chrono::TimeZone;
use hc_joining_code::Props;
// use holochain::conductor::api::error::{ConductorApiError, ConductorApiResult};
use holochain::sweettest::*;
use holochain_types::prelude::DnaFile;
use proptest::{prelude::*, test_runner::TestRunner};

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
    message_history: Vec<InsertFakeMessagePayload>,
    index: usize,
}

prop_compose! {
    fn generate_test_input(length: usize)(
        message_history in generate_message_history(length),
        index in (0..length)
    ) -> TestInput {
        TestInput {
            message_history,
            index,
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_batching() {
    // Use prebuilt DNA bundle.
    // You must build the DNA bundle as a separate step before running the test.
    let dna_path = std::env::current_dir()
        .unwrap()
        .join("../../elemental-chat.dna");

    // Convert to DnaFile and apply property overrides
    let dna = SweetDnaFile::from_bundle_with_overrides(
        &dna_path,
        Some(format!("test-{}", chrono::Utc::now().to_rfc3339())),
        // Note that we can use our own native `Props` type
        Some(Props {
            skip_proof: true,
            holo_agent_override: None,
            development_stage: None,
            t_and_c: None,
            t_and_c_agreement: None,
        }),
    )
    .await
    .unwrap();

    let shared_test_state = SharedTestState::new(dna).await;

    struct SharedTestState {
        conductor: SweetConductor,
        alice_chat: SweetZome,
        next_channel: AtomicU32,
    }

    impl SharedTestState {
        async fn new(dna: DnaFile) -> Self {
            // Set up conductor
            let mut conductor = SweetConductor::from_standard_config().await;

            let agents = SweetAgents::get(conductor.keystore(), 1).await;

            // Install apps with single DNA
            let apps = conductor
                .setup_app_for_agents("elemental-chat", &agents, &[dna.into()])
                .await
                .unwrap();
            let ((alice_cell,),) = apps.into_tuples();
            let alice_chat = alice_cell.zome("chat");

            // Setup complete.
            SharedTestState {
                conductor,
                alice_chat,
                next_channel: AtomicU32::new(0),
            }
        }

        fn next_channel_name(&self) -> String {
            let channel_idx = self.next_channel.fetch_add(1, SeqCst);
            format!("Test #{}", channel_idx)
        }

        async fn run(&self, _test_input: TestInput) {
            let _channel: ChannelData = self
                .conductor
                .call(
                    &self.alice_chat,
                    "create_channel",
                    ChannelInput {
                        name: self.next_channel_name(),
                        entry: Channel {
                            category: "General".into(),
                            uuid: uuid::Uuid::new_v4().to_string(),
                        },
                    },
                )
                .await;

            // insert_fake_messages(test_input.message_history);
            // for message in message_history {
            //     // Insert a message into the DHT at an arbitrary timestamp
            //     zome_call("insert_fake_message", message)
            // }

            // let earliest_seen = message_history[index];
            // let messages = timeout(zome_call("list_messages", earliest_seen), ...);

            // assert!(messages.all(|m| m.timestamp < earliest_seen.timestamp));
            // assert!(messages.is_sorted());
            // Ok(())

            // let _: () = conductor.call(alice_chat, "create_test_messages", message::handlers::InsertTestMessages {
            //     channel: channel.entry.clone(),
            //     number_of_messages: length.clone(),
            // }).await;
            // let time = Timestamp::now();
            // let lmpi = ListMessagesInput {
            //     channel: channel.entry.clone(),
            //     earliest_seen: Some(time),
            //     target_message_count: length as usize + 2,
            // };

            // let alice_msgs: ListMessages = conductor
            // .call(alice_chat, "list_messages", lmpi.clone())
            // .await;

            // assert_eq!(
            //     alice_msgs.messages.len(),
            //     length as usize + 2
            // );

            // assert!(alice_msgs.messages.iter().all(|m| m.created_at < time));
        }
    }

    let shared_test_state = Arc::new(shared_test_state);

    let test_result = tokio::task::spawn_blocking({
        let shared_test_state = Arc::clone(&shared_test_state);
        move || {
            let mut runner =
                TestRunner::new(proptest::test_runner::Config::with_source_file(file!()));
            runner.run(&generate_test_input(90), move |test_input| {
                tokio::runtime::Handle::current().block_on(shared_test_state.run(test_input));
                Ok(())
            })
        }
    })
    .await
    .unwrap();

    Arc::try_unwrap(shared_test_state)
        .unwrap_or_else(|_| panic!("shared test state should not have outstanding references because test has completed"))
        .conductor
        .shutdown()
        .await;

    test_result.unwrap();
}
