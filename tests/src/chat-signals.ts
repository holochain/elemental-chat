import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { RETRY_DELAY, RETRY_COUNT, localConductorConfig, networkedConductorConfig, installation1agent, installation2agent, installAgents } from './common'

const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = async (orchestrator) => {

  orchestrator.registerScenario('test signal', async (s, t) => {
    const config = localConductorConfig;

    const [alice, bob] = await s.players([config, config], false)
    await alice.startup()
    await bob.startup()
    let MESSAGE = {
      uuid: uuidv4(),
      content: "Hello from alice :)",
    }
    let receivedCount = 0
    bob.setSignalHandler((signal) => {
      console.log("Received Signal:",signal)
      t.deepEqual(signal.data.payload.signal_payload.messageData.entry, MESSAGE)
      receivedCount += 1
    })
    // const [[alice_chat_happ]] = await alice.installAgentsHapps(installation1agent)
    // const [[bob_chat_happ]] = await bob.installAgentsHapps(installation1agent)
    // const [alice_chat] = alice_chat_happ.cells
    // const [bob_chat] = bob_chat_happ.cells
    let [alice_chat_happ] = await installAgents(alice,  ["alice"])
    let [bob_chat_happ] = await installAgents(bob,  ['bob'])

    const [alice_chat] = alice_chat_happ.cells
    const [bob_chat] = bob_chat_happ.cells


    await s.shareAllNodes([alice, bob]);

    let stats = await alice_chat.call('chat', 'stats', {category: "General"});
    t.deepEqual(stats, {agents: 0, active: 0, channels: 0, messages: 0});

    // bob declares self as chatter
    await bob_chat.call('chat', 'refresh_chatter', null);
    // alice declares self as chatter
    await alice_chat.call('chat', 'refresh_chatter', null);

    // Create a channel
    const channel_uuid = uuidv4();
    const channel = await alice_chat.call('chat', 'create_channel', { name: "Test Channel", entry: { category: "General", uuid: channel_uuid } });
    console.log("CHANNEL: >>>", channel);

    const msg1 = {
      last_seen: { First: null },
      channel: channel.entry,
      chunk: 0,
      entry: MESSAGE
    }
    const r1 = await alice_chat.call('chat', 'create_message', msg1);
    t.deepEqual(r1.entry, msg1.entry);

    const signalMessageData = {
      messageData: r1,
      channelData: channel,
    };

    const r4 = await alice_chat.call('chat', 'signal_chatters', signalMessageData);
    t.equal(r4.total, 2)
    t.equal(r4.sent.length, 1)

    // waiting for the signal to be received by bob.
    for (let i = 0; i < 5; i++) {
      if (receivedCount > 0) break;
      console.log(`waiting for signal: ${i}`)
      await delay(500)
    }
    // bob should have gotten a signal becayse he's an active chatter
    t.equal(receivedCount, 1)

    stats = await alice_chat.call('chat', 'stats', {category: "General"});
    t.deepEqual(stats, {agents: 2, active: 2, channels: 1, messages: 1});

    await alice_chat.call('chat', 'signal_specific_chatters', {
      signal_message_data: signalMessageData,
      chatters: [bob_chat.cellId[1]]
    })

    // waiting for the signal to be received by bob.
    for (let i = 0; i < 5; i++) {
      if (receivedCount > 1) break;
      console.log(`waiting for signal: ${i}`)
      await delay(500)
    }
    // bob should have gotten a 2nd signal because he's specified in the call
    t.equal(receivedCount, 2)

    const result = await alice_chat.call('chat', 'get_active_chatters');
    t.equal(result.chatters.length, 1)
    t.equal(result.chatters[0].toString('base64'), bob_chat.cellId[1].toString('base64'))

    await alice_chat.call('chat', 'signal_specific_chatters', {
      signal_message_data: signalMessageData,
      chatters: [],
      include_active_chatters: false
    })

    // waiting for the signal to be received by bob.
    for (let i = 0; i < 5; i++) {
      if (receivedCount > 2) break;
      console.log(`waiting for signal: ${i}`)
      await delay(500)
    }
    // bob should NOT have gotten a 3rd signal because he's not specified in the call

    t.equal(receivedCount, 2)

    await alice_chat.call('chat', 'signal_specific_chatters', {
      signal_message_data: signalMessageData,
      chatters: [],
      include_active_chatters: true
    })

    // waiting for the signal to be received by bob.
    for (let i = 0; i < 5; i++) {
      if (receivedCount > 2) break;
      console.log(`waiting for signal: ${i}`)
      await delay(500)
    }
    // bob should now have gotten a 3rd signal because he's an active chatter and we included active chatters
    t.equal(receivedCount, 3)
  })
}
