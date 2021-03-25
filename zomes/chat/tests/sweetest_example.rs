/*use chat::{channel::Channel, ChannelData, ChannelInput, ChannelList, ChannelListInput};
use holochain::test_utils::sweetest::*;

#[tokio::test(threaded_scheduler)]
async fn sweetest_example() {
    // Use prebuilt DNA file
    let dna_path = std::env::current_dir()
        .unwrap()
        .join("../../elemental-chat.dna");
    let dna = SweetDnaFile::from_file(&dna_path).await.unwrap();

    // Set up conductor
    let mut conductor = SweetConductor::from_standard_config().await;
    let app = conductor.setup_app("elemental-chat", &[dna]).await;
    let zome = &app.cells()[0].zome("chat");

    // Run simple test
    let channel = Channel {
        category: "category".into(),
        uuid: "uuid".into(),
    };
    let _data: ChannelData = conductor
        .call(
            zome,
            "create_channel",
            ChannelInput {
                name: "name".into(),
                channel: channel.clone(),
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
    assert_eq!(channels.channels[0].channel, channel);
}
*/
