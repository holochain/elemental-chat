import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { RETRY_DELAY, RETRY_COUNT, localConductorConfig, networkedConductorConfig, awaitIntegration, delay } from './common'
import { installAgents } from './installAgents'

module.exports = async (orchestrator) => {
    orchestrator.registerScenario('transient nodes-local', async (s, t) => {
        await doTransientNodes(s, t, true)
    })
    /* Restore when tryorama double registerScenario bug fixed
    orchestrator.registerScenario('transient nodes-proxied', async (s, t) => {
        await doTransientNodes(s, t, false)
    })*/
}

const gotChannelsAndMessages = async(t, name, happ, channelEntry, retry_count, retry_delay)  => {
  var retries = retry_count
  while (true) {
    const channel_list = await happ.call('chat', 'list_channels', { category: "General" });
    console.log(`${name}'s channel list:`, channel_list.channels);
    let batch_payload = { channel: channelEntry, active_chatter: false, target_message_count: 2 }

    const r = await happ.call('chat', 'list_messages', batch_payload)
    t.ok(r)
    console.log(`${name}'s message list:`, r);
    if (r.messages.length > 0) {
      t.equal(r.messages.length,1)
      break;
    }
    else {
      retries -= 1;
      if (retries == 0) {
        t.fail(`bailing after ${retry_count} retries waiting for ${name}`)
        break;
      }
    }
    console.log(`retry ${retries}`);
    await delay( retry_delay )
  }
}



const doTransientNodes = async (s, t, local) => {
  const config = local ? localConductorConfig : networkedConductorConfig;

  const [alice, bob, carol] = await s.players([config, config, config], false)
  await alice.startup()
  await bob.startup()



  const [alice_chat_happ] = await installAgents(alice,  ["alice"])
  const [bob_chat_happ] = await installAgents(bob,  ['bobbo'])
  const [alice_chat] = alice_chat_happ.cells
  const [bob_chat] = bob_chat_happ.cells

  if (local) {
    await s.shareAllNodes([alice, bob]);
  }

  // Create a channel
  const channel_uuid = uuidv4();
  const channel = await alice_chat.call('chat', 'create_channel', { name: "Test Channel", entry: { category: "General", uuid: channel_uuid } });

  const msg1 = {
    last_seen: { First: null },
    channel: channel.entry,
    entry: {
      uuid: uuidv4(),
      content: "Hello from alice :)",
    }
  }
  const r1 = await alice_chat.call('chat', 'create_message', msg1);
  t.deepEqual(r1.entry, msg1.entry);

  console.log("******************************************************************")
  console.log("checking to see if bob can see the message")
  await gotChannelsAndMessages(t, "bob", bob_chat, channel.entry, RETRY_COUNT, RETRY_DELAY)
  console.log("waiting for bob to integrate the message not just see it via get")
  await awaitIntegration(bob_chat)
  await delay(10000)
  console.log("shutting down alice")
  await alice.shutdown()
  await delay(10000)
  console.log("checking again to see if bob can see the message")
  await gotChannelsAndMessages(t, "bob", bob_chat, channel.entry, RETRY_COUNT, RETRY_DELAY)
  await carol.startup()
  let [carol_chat_happ] = await installAgents(carol, ["carol"])
  const [carol_chat] = carol_chat_happ.cells

  if (local) {
    await s.shareAllNodes([carol, bob]);
  }

  console.log("******************************************************************")
  console.log("checking to see if carol can see the message via bob")
  await gotChannelsAndMessages(t, "carol", carol_chat, channel.entry, RETRY_COUNT, RETRY_DELAY)

  // This above loop SHOULD work because carol should get the message via bob, but it doesn't
  // So we try starting up alice and getting the message gossiped that way, but that also
  // doesn't work!
  await alice.startup()
  if (local) {
    await s.shareAllNodes([carol, alice]);
  }
  console.log("******************************************************************")
  console.log("checking to see if carol can see the message via alice after back on")
  await gotChannelsAndMessages(t, "carol", carol_chat, channel.entry, RETRY_COUNT, RETRY_DELAY)

}
