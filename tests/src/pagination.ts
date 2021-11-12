import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { localConductorConfig, delay, awaitIntegration } from './common'
import { installAgents } from './installAgents'

module.exports = async (orchestrator) => {

  orchestrator.registerScenario('chat away', async (s, t) => {
    // Declare two players using the previously specified config, nicknaming them "alice" and "bob"
    // note that the first argument to players is just an array conductor configs that that will
    // be used to spin up the conductor processes which are returned in a matching array.
    const [a_and_b_conductor] = await s.players([localConductorConfig])

    // install your happs into the coductors and destructuring the returned happ data using the same
    // array structure as you created in your installation array.
    let [alice_chat_happ, bobbo_chat_happ] = await installAgents(a_and_b_conductor,  ["alice", 'bobbo'])
    const [alice_chat] = alice_chat_happ.cells
    const [bobbo_chat] = bobbo_chat_happ.cells


    // Create a channel
    const channel_uuid = uuidv4();
    const channel = await alice_chat.call('chat', 'create_channel', { name: "Test Channel", entry: { category: "General", uuid: channel_uuid } });
    console.log(channel);

    var sends: any[] = [];
    var recvs: any[] = [];

    let first_message = {
      last_seen: { First: null },
      channel: channel.entry,
      chunk: 0,
      entry: {
        uuid: uuidv4(),
        content: "Hello from alice :)",
      }
    };

    // Alice send a message
    sends.push(first_message);
    console.log(sends[0]);

    recvs.push(await alice_chat.call('chat', 'create_message', sends[0]));
    console.log(recvs[0]);
    t.deepEqual(sends[0].entry, recvs[0].entry);

    // Alice sends another message
    sends.push({
      last_seen: { Message: recvs[0].entryHash },
      channel: channel.entry,
      chunk: 0,
      entry: {
        uuid: uuidv4(),
        content: "Is anybody out there?",
      }
    });
    await delay(10000)
    console.log(sends[1]);
    recvs.push(await alice_chat.call('chat', 'create_message', sends[1]));
    console.log(recvs[1]);
    t.deepEqual(sends[1].entry, recvs[1].entry);

    await delay(10000)    
    await awaitIntegration(bobbo_chat)

    // Alice lists the messages
    let alices_view = await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, target_message_count: 2 })
    
    let bobbos_view = await bobbo_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, target_message_count: 2 })
    
    if (alices_view.messages.length !== 2) {
      await delay(10000)
      console.log("Trying again...");
      
      alices_view = await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, target_message_count: 2 })
   
      bobbos_view = await bobbo_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, target_message_count: 2 })     
    }
    t.deepEqual(alices_view.messages.length, 2)
    t.deepEqual(bobbos_view.messages.length, 2)
    // Bobbo and Alice both reply to the same message
    sends.push({
      last_seen: { Message: recvs[1].entryHash },
      channel: channel.entry,
      chunk: 0,
      entry: {
        uuid: uuidv4(),
        content: "I'm here",
      }
    });
    sends.push({
      last_seen: { Message: recvs[1].entryHash },
      channel: channel.entry,
      chunk: 0,
      entry: {
        uuid: uuidv4(),
        content: "Anybody?",
      }
    });
    recvs.push(await bobbo_chat.call('chat', 'create_message', sends[2]));
    console.log(recvs[2]);
    t.deepEqual(sends[2].entry, recvs[2].entry);
    recvs.push(await alice_chat.call('chat', 'create_message', sends[3]));
    console.log(recvs[3]);
    t.deepEqual(sends[3].entry, recvs[3].entry);
    await delay(4000)
    // Alice lists the messages
    alices_view = await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, target_message_count: 2 })
    // Bobbo lists the messages
    bobbos_view = await bobbo_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, target_message_count: 2 })
    t.deepEqual(alices_view.messages.length, 4)
    t.deepEqual(bobbos_view.messages.length, 4)
    console.log("ALICE ", alices_view);
    console.log("BOBBO: ", bobbos_view);
    
  })

}
