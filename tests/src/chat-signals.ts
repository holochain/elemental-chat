import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { RETRY_DELAY, RETRY_COUNT, conductorConfig, networkedConductorConfig, installation1agent, installation2agent } from './common'

const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = async (orchestrator) => {

  orchestrator.registerScenario.only('test signal', async (s, t) => {
    const config = networkedConductorConfig;

    const [alice, bob] = await s.players([config, config], false)
    await alice.startup()
    await bob.startup()
    let MESSAGE = {
      uuid: uuidv4(),
      content: "Hello from alice :)",
    }
    let flag = false
    bob.setSignalHandler((signal) => {
        console.log("Received Signal:",signal)
        t.deepEqual(signal.data.payload.signal_payload.messageData.message, MESSAGE)
        flag = true
    })
    const [[alice_chat_happ]] = await alice.installAgentsHapps(installation1agent)
    const [[bob_chat_happ]] = await bob.installAgentsHapps(installation1agent)
    const [alice_chat] = alice_chat_happ.cells
    const [bob_chat] = bob_chat_happ.cells

    await s.shareAllNodes([alice, bob]);

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

    await delay(5000)

    await alice_chat.call('chat', 'refresh_chatter', null);

    await bob_chat.call('chat', 'refresh_chatter', null);
    await delay(2000)
    const signalMessageData = {
      messageData: r1,
      channelData: channel,
    };
    const r4 = await alice_chat.call('chat', 'signal_chatters', signalMessageData);
    console.log("-->", r4);
    t.equal(r4.total, 2)

    // waiting for the signal to be received by bob. 
    for (let i = 0; i < 5; i++) {
      if (flag) break;
      await delay(10000)
    }
    t.ok(flag)
  })
}
