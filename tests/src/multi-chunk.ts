import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { localConductorConfig, networkedConductorConfig, installation1agent, installation2agent } from './common'

const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = async (orchestrator) => {

  orchestrator.registerScenario('multi-chunk', async (s, t) => {
    const [conductor] = await s.players([localConductorConfig])
    const [
      [alice_chat_happ],
    ] = await conductor.installAgentsHapps(installation1agent)
    const [alice_chat] = alice_chat_happ.cells

    const channel_uuid = uuidv4();
    const channel = await alice_chat.call('chat', 'create_channel', { name: "Test Channel", entry: { category: "General", uuid: channel_uuid } });
    console.log(channel);

    let channel_list = await alice_chat.call('chat', 'list_channels', { category: "General" });
    t.deepEqual(channel, channel_list.channels[0]);
    t.equal(channel_list.channels[0].latestChunk, 0);

    var sends: any[] = [];
    var recvs: any[] = [];

    // Alice send a message in two different chunks
    sends.push({
      last_seen: { First: null },
      channel: channel.entry,
      chunk: 0,
      entry: {
        uuid: uuidv4(),
        content: "message in chunk 0",
      }
    });

    recvs.push(await alice_chat.call('chat', 'create_message', sends[0]));
    sends.push({
      last_seen: { First: null },
      channel: channel.entry,
      chunk: 10,
      entry: {
        uuid: uuidv4(),
        content: "message in chunk 32",
      }
    });

    recvs.push(await alice_chat.call('chat', 'create_message', sends[1]));
    t.deepEqual(sends[0].message, recvs[0].message);

    // list messages should return messages from the correct chunk
    let msgs = await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, chunk: {start:0, end: 1} })
    t.deepEqual(msgs.messages[0].message, sends[0].message)
    msgs = await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, chunk: {start:1, end: 1} })
    t.equal(msgs.messages.length, 0)
    msgs = await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, chunk: {start:10, end: 10} })
    t.deepEqual(msgs.messages[0].message, sends[1].message)
    msgs = await alice_chat.call('chat', 'list_messages', { channel: channel.entry, active_chatter: false, chunk: {start:0, end: 10} })
    t.deepEqual(msgs.messages.length, 2)

    // list channels should have the latest chunk
    channel_list = await alice_chat.call('chat', 'list_channels', { category: "General" });
    t.equal(channel_list.channels[0].latestChunk, 10);
  })
}
