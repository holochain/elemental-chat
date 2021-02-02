import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { RETRY_DELAY, RETRY_COUNT, localConductorConfig, networkedConductorConfig, installation1agent, installation2agent } from './common'

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
      t.deepEqual(signal.data.payload.signal_payload.messageData.message, MESSAGE)
      receivedCount += 1
    })
    const [[alice_chat_happ]] = await alice.installAgentsHapps(installation1agent)
    const [[bob_chat_happ]] = await bob.installAgentsHapps(installation1agent)
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
    const channel = await alice_chat.call('chat', 'create_channel', { name: "Test Channel", channel: { category: "General", uuid: channel_uuid } });
    console.log("CHANNEL: >>>", channel);

    const msg1 = {
      last_seen: { First: null },
      channel: channel.channel,
      chunk: 0,
      message: MESSAGE
    }
    const r1 = await alice_chat.call('chat', 'create_message', msg1);
    t.deepEqual(r1.message, msg1.message);

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
      console.log(`wating for signal: ${i}`)
      await delay(500)
    }
    t.equal(receivedCount, 1)

    stats = await alice_chat.call('chat', 'stats', {category: "General"});
    t.deepEqual(stats, {agents: 2, active: 2, channels: 1, messages: 1});

    const r5 = await alice_chat.call('chat', 'signal_specific_chatters', {
      signal_message_data: signalMessageData,
      chatters: [bob_chat.cellId[1]]
    }
    );

    // waiting for the signal to be received by bob.
    for (let i = 0; i < 5; i++) {
      if (receivedCount > 1) break;
      console.log(`wating for signal: ${i}`)
      await delay(500)
    }
    t.equal(receivedCount, 2)
  })
}
