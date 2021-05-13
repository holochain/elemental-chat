import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { RETRY_DELAY, RETRY_COUNT, localConductorConfig, networkedConductorConfig, installAgents, MEM_PROOF_BAD_SIG, MEM_PROOF1, MEM_PROOF2, awaitIntegration, delay } from './common'

module.exports = async (orchestrator) => {

  orchestrator.registerScenario('chat away', async (s, t) => {
    // Declare two players using the previously specified config, nicknaming them "alice" and "bob"
    // note that the first argument to players is just an array conductor configs that that will
    // be used to spin up the conductor processes which are returned in a matching array.
    const [a_and_b_conductor] = await s.players([localConductorConfig])

    // install your happs into the coductors and destructuring the returned happ data using the same
    // array structure as you created in your installation array.
    let [alice_chat_happ, bobbo_chat_happ] = await installAgents(a_and_b_conductor,  ["alice", 'bobbo'], [MEM_PROOF1,  MEM_PROOF2])
    const [alice_chat] = alice_chat_happ.cells
    const [bobbo_chat] = bobbo_chat_happ.cells


    // Create a channel
    const channel_uuid = uuidv4();
    const channel = await alice_chat.call('chat', 'create_channel', { name: "Test Channel", entry: { category: "General", uuid: channel_uuid } });
    console.log(channel);

    var sends: any[] = [];
    var recvs: any[] = [];
    function messageEntry(m) { return m.entry }

    let first_message = {
      last_seen: { First: null },
      channel: channel.entry,
      chunk: 0,
      entry: {
        uuid: uuidv4(),
        content: 'x'.repeat(1025),
      }
    };

    //Send a messages that's too long
    try {
      await alice_chat.call('chat', 'create_message', first_message);
      t.fail()
    } catch(e) {
      t.deepEqual(e,{ type: 'error', data: { type: 'internal_error', data: 'Source chain error: InvalidCommit error: Message too long' } })
    }

    first_message.entry.content = "Hello from alice :)";
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
    console.log(sends[1]);
    recvs.push(await alice_chat.call('chat', 'create_message', sends[1]));
    console.log(recvs[1]);
    t.deepEqual(sends[1].entry, recvs[1].entry);

    const channel_list = await alice_chat.call('chat', 'list_channels', { category: "General" });
    console.log(channel_list);

    // Alice lists the messages
    var msgs: any[] = [];
    msgs.push(await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, chunk: {start:0, end: 1} }));
    console.log(_.map(msgs[0].messages, messageEntry));
    t.deepEqual([sends[0].entry, sends[1].entry], _.map(msgs[0].messages, messageEntry));
    // Bobbo lists the messages
    await delay(2000) // TODO add consistency instead
    msgs.push(await bobbo_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, chunk: {start:0, end: 1} }));
    console.log('bobbo.list_messages: '+_.map(msgs[1].messages, messageEntry));
    t.deepEqual([sends[0].entry, sends[1].entry], _.map(msgs[1].messages, messageEntry));

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
    msgs.push(await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, chunk: {start:0, end: 1} }));
    console.log(_.map(msgs[2].messages, messageEntry));
    t.deepEqual([sends[0].entry, sends[1].entry, sends[2].entry, sends[3].entry], _.map(msgs[2].messages, messageEntry));
    // Bobbo lists the messages
    msgs.push(await bobbo_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, chunk: {start:0, end: 1} }));
    console.log(_.map(msgs[3].messages, messageEntry));
    t.deepEqual([sends[0].entry, sends[1].entry, sends[2].entry, sends[3].entry], _.map(msgs[3].messages, messageEntry));

    const allMessages = await bobbo_chat.call('chat', 'list_all_messages', { category: "General", chunk: {start:0, end: 1} })
    t.equal(allMessages[0].channel.info.name, "Test Channel");
    t.deepEqual(allMessages[0].messages.length, 4);
  })

}
