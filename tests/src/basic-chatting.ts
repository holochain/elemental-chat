import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { RETRY_DELAY, RETRY_COUNT, localConductorConfig, networkedConductorConfig, installation1agent, installation2agent } from './common'

const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = async (orchestrator) => {

   orchestrator.registerScenario('chat away', async (s, t) => {
    // Declare two players using the previously specified config, nicknaming them "alice" and "bob"
    // note that the first argument to players is just an array conductor configs that that will
    // be used to spin up the conductor processes which are returned in a matching array.
    const [a_and_b_conductor] = await s.players([localConductorConfig])

    // install your happs into the coductors and destructuring the returned happ data using the same
    // array structure as you created in your installation array.
    const [
	[alice_chat_happ],
	[bobbo_chat_happ],
    ] = await a_and_b_conductor.installAgentsHapps(installation2agent)
    const [alice_chat] = alice_chat_happ.cells
    const [bobbo_chat] = bobbo_chat_happ.cells

    // Create a channel
    const channel_uuid = uuidv4();
    const channel = await alice_chat.call('chat', 'create_channel', { name: "Test Channel", channel: { category: "General", uuid: channel_uuid } });
    console.log(channel);

    var sends: any[] = [];
    var recvs: any[] = [];
    function just_msg(m) { return m.message }

    let first_message = {
      last_seen: { First: null },
      channel: channel.channel,
      chunk: 0,
      message: {
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

    first_message.message.content = "Hello from alice :)";
    // Alice send a message
    sends.push(first_message);
    console.log(sends[0]);

    recvs.push(await alice_chat.call('chat', 'create_message', sends[0]));
    console.log(recvs[0]);
    t.deepEqual(sends[0].message, recvs[0].message);

    // Alice sends another message
    sends.push({
      last_seen: { Message: recvs[0].entryHash },
      channel: channel.channel,
      chunk: 0,
      message: {
        uuid: uuidv4(),
        content: "Is anybody out there?",
      }
    });
    console.log(sends[1]);
    recvs.push(await alice_chat.call('chat', 'create_message', sends[1]));
    console.log(recvs[1]);
    t.deepEqual(sends[1].message, recvs[1].message);

    const channel_list = await alice_chat.call('chat', 'list_channels', { category: "General" });
    console.log(channel_list);

    // Alice lists the messages
    var msgs: any[] = [];
    msgs.push(await alice_chat.call('chat', 'list_messages', { channel: channel.channel, active_chatter: false, chunk: {start:0, end: 1} }));
    console.log(_.map(msgs[0].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message], _.map(msgs[0].messages, just_msg));
    // Bobbo lists the messages
    await delay(2000) // TODO add consistency instead
    msgs.push(await bobbo_chat.call('chat', 'list_messages', { channel: channel.channel, active_chatter: false, chunk: {start:0, end: 1} }));
    console.log('bobbo.list_messages: '+_.map(msgs[1].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message], _.map(msgs[1].messages, just_msg));

    // Bobbo and Alice both reply to the same message
    sends.push({
      last_seen: { Message: recvs[1].entryHash },
      channel: channel.channel,
      chunk: 0,
      message: {
        uuid: uuidv4(),
        content: "I'm here",
      }
    });
    sends.push({
      last_seen: { Message: recvs[1].entryHash },
      channel: channel.channel,
      chunk: 0,
      message: {
        uuid: uuidv4(),
        content: "Anybody?",
      }
    });
    recvs.push(await bobbo_chat.call('chat', 'create_message', sends[2]));
    console.log(recvs[2]);
    t.deepEqual(sends[2].message, recvs[2].message);
    recvs.push(await alice_chat.call('chat', 'create_message', sends[3]));
    console.log(recvs[3]);
    t.deepEqual(sends[3].message, recvs[3].message);
    await delay(4000)
    // Alice lists the messages
    msgs.push(await alice_chat.call('chat', 'list_messages', { channel: channel.channel, active_chatter: false, chunk: {start:0, end: 1} }));
    console.log(_.map(msgs[2].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message, sends[2].message, sends[3].message], _.map(msgs[2].messages, just_msg));
    // Bobbo lists the messages
    msgs.push(await bobbo_chat.call('chat', 'list_messages', { channel: channel.channel, active_chatter: false, chunk: {start:0, end: 1} }));
    console.log(_.map(msgs[3].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message, sends[2].message, sends[3].message], _.map(msgs[3].messages, just_msg));
  })

}
