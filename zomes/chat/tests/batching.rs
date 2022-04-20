use std::{
    sync::{
        atomic::{AtomicU32, Ordering::SeqCst},
        Arc,
    },
    time::Duration,
};

use chat::{
    message::handlers::{FakeMessage, InsertFakeMessagesPayload},
    Channel, ChannelData, ChannelInput, ListMessages, ListMessagesInput, Timestamp,
};

use chrono::{DateTime, TimeZone, Timelike};
use hc_joining_code::Props;
use holochain::sweettest::*;
use holochain_types::prelude::DnaFile;
use proptest::{prelude::*, test_runner::TestRunner};

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
    fn generate_message_history()(
        timestamps in prop::collection::vec(generate_timestamp(), 1..30)
    ) -> Vec<FakeMessage> {
        timestamps.into_iter().enumerate().map(|(i, timestamp)| FakeMessage { content: format!("{}", i), timestamp}).collect()
    }
}

#[derive(Clone, Debug)]
struct TestInput {
    message_history: Vec<FakeMessage>,
    earliest_seen: Timestamp,
    target_message_count: usize,
}

prop_compose! {
    fn generate_test_input()(message_history in generate_message_history())
        (
            index in (0..message_history.len()), message_history in Just(message_history)
        )
     -> TestInput {
        TestInput {
            earliest_seen: message_history[index].timestamp,
            message_history,
            target_message_count: 20,
        }
    }
}

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
            .setup_app_for_agents("elemental-chat", &agents, &[dna])
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

    async fn run(&self, test_input: TestInput) {
        let channel: ChannelData = self
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

        // Insert messages with artificial timestamps into the DHT
        let _: () = self
            .conductor
            .call(
                &self.alice_chat,
                "insert_fake_messages",
                InsertFakeMessagesPayload {
                    messages: test_input.message_history.clone(),
                    channel: channel.entry.clone(),
                },
            )
            .await;

        let ListMessages { messages } = tokio::time::timeout(
            Duration::from_millis(15_000),
            self.conductor.call(
                &self.alice_chat,
                "list_messages",
                ListMessagesInput {
                    channel: channel.entry,
                    earliest_seen: Some(test_input.earliest_seen),
                    target_message_count: test_input.target_message_count,
                },
            ),
        )
        .await
        .unwrap();

        let mut messages: Vec<_> = messages.into_iter().map(|m| m.entry.content).collect();
        messages.sort_unstable();

        let mut expected = expected_messages(test_input.clone());
        expected.sort_unstable();

        assert_eq!(
            messages, expected,
            "(returned messages) == (expected_messages). input: {:?}",
            test_input
        );
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

    let shared_test_state = Arc::new(shared_test_state);

    let test_result = tokio::task::spawn_blocking({
        let shared_test_state = Arc::clone(&shared_test_state);
        move || {
            let mut runner = TestRunner::new(proptest::test_runner::Config {
                // Make sure that proptest knows the source location so it knows where to put the .regressions file
                source_file: Some(file!()),
                // Limit how long the test takes (each iteration takes ~1 second)
                cases: 10,
                max_shrink_time: 1000 * 20,
                ..Default::default()
            });
            runner.run(&generate_test_input(), move |test_input| {
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

fn expected_messages(test_input: TestInput) -> Vec<String> {
    fn same_hour(a: &Timestamp, b: &Timestamp) -> bool {
        let a = DateTime::try_from(a).unwrap();
        let b = DateTime::try_from(b).unwrap();
        a.signed_duration_since(b).num_hours() == 0 && a.time().hour() == b.time().hour()
    }

    let TestInput {
        message_history: mut messages,
        earliest_seen,
        target_message_count,
    } = test_input;
    messages.retain(|m| m.timestamp < earliest_seen);
    messages.sort_unstable_by_key(|m| m.timestamp);
    let (only_included_if_same_hour, included) =
        messages.split_at(messages.len().saturating_sub(target_message_count));
    let earliest_included_hour = if let Some(m) = included.first() {
        &m.timestamp
    } else {
        return Vec::new();
    };
    if let Some(different_hour_idx) = only_included_if_same_hour
        .iter()
        .rposition(|m| !same_hour(&m.timestamp, earliest_included_hour))
    {
        messages.drain(0..=different_hour_idx);
    }

    messages.into_iter().map(|m| m.content).collect()
}

#[test]
fn expected_messages_works() {
    assert_eq!(
        expected_messages(TestInput {
            message_history: vec![
                FakeMessage {
                    content: "0".to_owned(),
                    timestamp: Timestamp::from(chrono::Utc.ymd(2022, 1, 1).and_hms(0, 0, 0))
                },
                FakeMessage {
                    content: "1".to_owned(),
                    timestamp: Timestamp::from(chrono::Utc.ymd(2022, 1, 1).and_hms(0, 0, 0))
                }
            ],
            earliest_seen: Timestamp::from(chrono::Utc.ymd(2022, 1, 1).and_hms(0, 0, 0)),
            target_message_count: 1
        }),
        Vec::<String>::new(),
    );

    assert_eq!(
        expected_messages(TestInput {
            message_history: vec![
                FakeMessage {
                    content: "0".to_owned(),
                    timestamp: Timestamp::from(chrono::Utc.ymd(2022, 1, 2).and_hms(2, 0, 0)),
                },
                FakeMessage {
                    content: "1".to_owned(),
                    timestamp: Timestamp::from(chrono::Utc.ymd(2022, 1, 1).and_hms(1, 0, 0)),
                },
            ],
            earliest_seen: Timestamp::from(chrono::Utc.ymd(2022, 2, 1).and_hms(0, 0, 0)),
            target_message_count: 1
        }),
        vec!["0".to_owned()]
    );

    // TODO: The real implementation has a bug here
    assert_eq!(
        expected_messages(TestInput {
            message_history: vec![FakeMessage {
                content: "13".to_owned(),
                timestamp: Timestamp::from(chrono::Utc.ymd(2022, 3, 2).and_hms(0, 0, 0))
            }],
            earliest_seen: Timestamp::from(chrono::Utc.ymd(2022, 3, 2).and_hms(1, 0, 0)),
            target_message_count: 0,
        }),
        Vec::<String>::new()
    );
}
