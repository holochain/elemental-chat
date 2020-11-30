import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";

const delay = ms => new Promise(r => setTimeout(r, ms))

// Set up a Conductor configuration using the handy `Conductor.config` helper.
// Read the docs for more on configuration.
const conductorConfig = Config.gen()

// Construct proper paths for your DNAs
const chatDna = path.join(__dirname, "../../elemental-chat.dna.gz")

// create an InstallAgentsHapps array with your DNAs to tell tryorama what
// to install into the conductor.
const installation: InstallAgentsHapps = [
    // agent 0
    [
	// happ 0
	[chatDna]
    ],
    // agent 1
    [
	// happ 0
	[chatDna]
    ],
]


module.exports = (orchestrator) => {
  // This is placeholder for signals test; awaiting implementation of signals testing in tryorama.
  // Issue: https://github.com/holochain/tryorama/issues/40

  orchestrator.registerScenario.skip('emit signals', async (s, t) => {})

  orchestrator.registerScenario('chat away', async (s, t) => {
    // Declare two players using the previously specified config, nicknaming them "alice" and "bob"
    // note that the first argument to players is just an array conductor configs that that will
    // be used to spin up the conductor processes which are returned in a matching array.
    const [a_and_b_conductor] = await s.players([conductorConfig])

    // install your happs into the coductors and destructuring the returned happ data using the same
    // array structure as you created in your installation array.
    const [
	[alice_chat_happ],
	[bobbo_chat_happ],
    ] = await a_and_b_conductor.installAgentsHapps(installation)
    const [alice_chat] = alice_chat_happ.cells
    const [bobbo_chat] = bobbo_chat_happ.cells

    // Create a channel
    const channel_uuid = uuidv4();
    const channel = await alice_chat.call('chat', 'create_channel', { name: "Test Channel", channel: { category: "General", uuid: channel_uuid } });
    console.log(channel);

    var sends: any[] = [];
    var recvs: any[] = [];
    function just_msg(m) { return m.message }

    // Alice send a message
    sends.push({
      last_seen: { First: null },
      channel: channel.channel,
      chunk: 0,
      message: {
        uuid: uuidv4(),
        content: "Hello from alice :)",
      }
    });
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
    msgs.push(await alice_chat.call('chat', 'list_messages', { channel: channel.channel, chunk: 0 }));
    console.log(_.map(msgs[0].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message], _.map(msgs[0].messages, just_msg));
    // Bobbo lists the messages
    await delay( 1000 )
    msgs.push(await bobbo_chat.call('chat', 'list_messages', { channel: channel.channel, chunk: 0 }));
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

    // Alice lists the messages
    msgs.push(await alice_chat.call('chat', 'list_messages', { channel: channel.channel, chunk: 0 }));
    console.log(_.map(msgs[2].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message, sends[2].message, sends[3].message], _.map(msgs[2].messages, just_msg));
    // Bobbo lists the messages
    msgs.push(await bobbo_chat.call('chat', 'list_messages', { channel: channel.channel, chunk: 0 }));
    console.log(_.map(msgs[3].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message, sends[2].message, sends[3].message], _.map(msgs[3].messages, just_msg));
  })
}
