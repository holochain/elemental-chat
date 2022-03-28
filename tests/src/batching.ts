import * as _ from "lodash";
import { v4 as uuidv4 } from "uuid";
import { localConductorConfig, delay, awaitIntegration } from "./common";
import { installAgents } from "./installAgents";

module.exports = async (orchestrator) => {
  orchestrator.registerScenario("chat with batches", async (s, t) => {
    // Declare two players using the previously specified config, nicknaming them "alice" and "bob"
    // note that the first argument to players is just an array conductor configs that that will
    // be used to spin up the conductor processes which are returned in a matching array.
    const [a_and_b_conductor] = await s.players([localConductorConfig]);

    // install your happs into the coductors and destructuring the returned happ data using the same
    // array structure as you created in your installation array.
    let [alice_chat_happ, bobbo_chat_happ] = await installAgents(
      a_and_b_conductor,
      ["alice", "bobbo"]
    );
    const [alice_chat] = alice_chat_happ.cells;
    const [bobbo_chat] = bobbo_chat_happ.cells;

    // Create a channel
    const channel_uuid = uuidv4();
    const channel = await alice_chat.call("chat", "create_channel", {
      name: "Test Channel",
      entry: { category: "General", uuid: channel_uuid },
    });
    console.log(channel);

    var sends: any[] = [];
    var recvs: any[] = [];

    let message = {
      last_seen: { First: null },
      channel: channel.entry,
      entry: {
        uuid: uuidv4(),
        content: "",
      },
    };
    /*
    // Alice send 5 messages
    for (let i=1 ;i<=5; i++) {
      message.entry.content = `Hello ${i}` 
      recvs.push(await alice_chat.call('chat', 'create_message', message));
      await delay(500)
    }
    await delay(3000)
    // send 5 more
    for (let i=1 ;i<=5; i++) {
      message.entry.content = `Hello ${i+5}` 
      recvs.push(await alice_chat.call('chat', 'create_message', message));
    }*/

    recvs.push(
      await alice_chat.call("chat", "create_test_messages", {channel: channel.entry, number_of_messages: 10})
    );

    await awaitIntegration(alice_chat);
    await awaitIntegration(bobbo_chat);

    // Alice lists the messages
    let results = await alice_chat.call("chat", "list_messages", {
      channel: channel.entry,
      active_chatter: false,
      target_message_count: 1,
    });
    t.is(results.messages.length, 3); // because 3 are clustered in the last hour
    results = await alice_chat.call("chat", "list_messages", {
      channel: channel.entry,
      active_chatter: false,
      target_message_count: 3,
    });
    t.is(results.messages.length, 3);
    results = await alice_chat.call("chat", "list_messages", {
      channel: channel.entry,
      active_chatter: false,
      target_message_count: 5,
    });
    t.is(results.messages.length, 5);
    results = await alice_chat.call("chat", "list_messages", {
      channel: channel.entry,
      active_chatter: false,
      target_message_count: 10,
    });
    t.is(results.messages.length, 10);
    results = await alice_chat.call("chat", "list_messages", {
      channel: channel.entry,
      active_chatter: false,
      target_message_count: 12,
    });
    t.is(results.messages.length, 12);
    results = await alice_chat.call("chat", "list_messages", {
      channel: channel.entry,
      active_chatter: false,
      target_message_count: 100,
    });
    t.is(results.messages.length, 12);

    console.log(`got ${results.messages.length} messages`);
    //t.deepEqual(bobbos_view.messages.length, 10)
    //console.log("ALICE ", alices_view);
    //console.log("BOBBO: ", bobbos_view);
  });
};
