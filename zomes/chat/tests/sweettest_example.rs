use chat::{channel::Channel, ChannelData, ChannelInput, ChannelList, ChannelListInput};
use hc_joining_code::Props;
use holochain::sweettest::*;

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn sweettest_example() {
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
        }),
    )
    .await
    .unwrap();

    // Set up conductor
    let mut conductor = SweetConductor::from_standard_config().await;

    // Install app with single DNA
    let app = conductor.setup_app("elemental-chat", &[dna]).await.unwrap();
    let zome = &app.cells()[0].zome("chat");

    // Setup complete. Run a simple test.

    // Note that we can use the native types defined in our zome
    let channel = Channel {
        category: "category".into(),
        uuid: "uuid".into(),
    };

    // Make some zome calls.
    // Note again that we can use native types.
    // We still have to give a type annotation for the return value, because the types
    // are still being serialized/deserialized into and out of Wasm.
    let _data: ChannelData = conductor
        .call(
            zome,
            "create_channel",
            ChannelInput {
                name: "name".into(),
                entry: channel.clone(),
            },
        )
        .await;
    let channels: ChannelList = conductor
        .call(
            zome,
            "list_channels",
            ChannelListInput {
                category: "category".into(),
            },
        )
        .await;

    // Compare return values directly using the rust `Eq` implementation,
    // rather than deep comparison of data serialized to JSON
    assert_eq!(channels.channels[0].entry, channel);
}
