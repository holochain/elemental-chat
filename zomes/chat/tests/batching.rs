use chat::channel::*;
use chat::message::*;
use chat::*;
use hc_joining_code::Props;
// use holochain::conductor::api::error::{ConductorApiError, ConductorApiResult};
use holochain::sweettest::*;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]
    #[test]
    fn test_batching(length in 1i32..20) {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let _ = rt.block_on(async move {

                    // Use prebuilt DNA bundle.
                    // You must build the DNA bundle as a separate step before running the test.
                    let dna_path = std::env::current_dir()
                        .unwrap()
                        .join("../../elemental-chat.dna");

                    // Convert to DnaFile and apply property overrides
                    let dna = SweetDnaFile::from_bundle_with_overrides(
                        &dna_path,
                        None,
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

                    // Set up conductor
                    let mut conductor = SweetConductor::from_standard_config().await;

                    let agents = SweetAgents::get(conductor.keystore(), 2).await;

                    // Install apps with single DNA
                    let apps = conductor
                        .setup_app_for_agents("elemental-chat", &agents, &[dna])
                        .await
                        .unwrap();
                    let ((alice_cell,), (_bobbo_cell,)) = apps.into_tuples();
                    let alice_chat = &alice_cell.zome("chat");
                    // let bobbo_chat = &bobbo_cell.zome("chat");

                    // Setup complete.

                    let channel: ChannelData = conductor
                    .call(
                        alice_chat,
                        "create_channel",
                        ChannelInput {
                            name: "Test Ch".into(),
                            entry: Channel {
                                category: "General".into(),
                                uuid: uuid::Uuid::new_v4().to_string(),
                            },
                        },
                    )
                    .await;

                    let _: () = conductor.call(alice_chat, "create_test_messages", message::handlers::InsertTestMessages {
                        channel: channel.entry.clone(),
                        number_of_messages: length.clone(),
                    }).await;
                    let time = Timestamp::now();
                    let lmpi = ListMessagesInput {
                        channel: channel.entry.clone(),
                        earliest_seen: Some(time),
                        target_message_count: 100,
                    };

                    let alice_msgs: ListMessages = conductor
                    .call(alice_chat, "list_messages", lmpi.clone())
                    .await;

                    assert_eq!(
                        alice_msgs.messages.len(),
                        length as usize + 2
                    );

                    assert!(alice_msgs.messages.iter().all(|m| m.created_at < time));

                    conductor.shutdown().await;
                });
            }
            _ => println!("Cannot spin up async test")
        }

    }
}
