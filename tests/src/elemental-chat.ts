import { Config } from '@holochain/tryorama'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";

const delay = ms => new Promise(r => setTimeout(r, ms))

// Configure a conductor with two identical DNAs,
// differentiated by UUID, nicknamed "alice" and "bobbo"
const config = Config.gen({
  alice: Config.dna("../elemental-chat.dna.gz", null),
  bobbo: Config.dna("../elemental-chat.dna.gz", null),
})

module.exports = (orchestrator) => {

  orchestrator.registerScenario('chat away', async (s, t) => {
    // spawn the conductor process
    const { conductor } = await s.players({ conductor: config })
    await conductor.spawn()

    // Create a channel
    const channel_uuid = uuidv4();
    const channel = await conductor.call('alice', 'chat', 'create_channel', { name: "Test Channel", channel: { category: "General", uuid: channel_uuid } });
    console.log(channel);

    var sends: any[] = [];
    var recvs: any[] = [];
    function just_msg(m) { return m.message }

    // Alice send a message
    sends.push({
      last_seen: { First: null },
      channel: channel.channel,
      message: {
        uuid: uuidv4(),
        content: "Hello from alice :)",
      }
    });
    console.log(sends[0]);
    recvs.push(await conductor.call('alice', 'chat', 'create_message', sends[0]));
    console.log(recvs[0]);
    t.deepEqual(sends[0].message, recvs[0].message);

    // Alice sends another message
    sends.push({
      last_seen: { Message: recvs[0].entryHash },
      channel: channel.channel,
      message: {
        uuid: uuidv4(),
        content: "Is anybody out there?",
      }
    });
    console.log(sends[1]);
    recvs.push(await conductor.call('alice', 'chat', 'create_message', sends[1]));
    console.log(recvs[1]);
    t.deepEqual(sends[1].message, recvs[1].message);

    const channel_list = await conductor.call('alice', 'chat', 'list_channels', { category: "General" });
    console.log(channel_list);

    // Alice lists the messages
    var msgs: any[] = [];
    console.log(today());
    msgs.push(await conductor.call('alice', 'chat', 'list_messages', { channel: channel.channel, date: today() }));
    console.log(_.map(msgs[0].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message], _.map(msgs[0].messages, just_msg));
    // Bobbo lists the messages
    msgs.push(await conductor.call('bobbo', 'chat', 'list_messages', { channel: channel.channel, date: today() }));
    console.log(_.map(msgs[1].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message], _.map(msgs[1].messages, just_msg));

    // Bobbo and Alice both reply to the same message
    sends.push({
      last_seen: { Message: recvs[1].entryHash },
      channel: channel.channel,
      message: {
        uuid: uuidv4(),
        content: "I'm here",
      }
    });
    sends.push({
      last_seen: { Message: recvs[1].entryHash },
      channel: channel.channel,
      message: {
        uuid: uuidv4(),
        content: "Anybody?",
      }
    });
    recvs.push(await conductor.call('bobbo', 'chat', 'create_message', sends[2]));
    console.log(recvs[2]);
    t.deepEqual(sends[2].message, recvs[2].message);
    recvs.push(await conductor.call('alice', 'chat', 'create_message', sends[3]));
    console.log(recvs[3]);
    t.deepEqual(sends[3].message, recvs[3].message);

    // Alice lists the messages
    msgs.push(await conductor.call('alice', 'chat', 'list_messages', { channel: channel.channel, date: today() }));
    console.log(_.map(msgs[2].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message, sends[2].message, sends[3].message], _.map(msgs[2].messages, just_msg));
    // Bobbo lists the messages
    msgs.push(await conductor.call('bobbo', 'chat', 'list_messages', { channel: channel.channel, date: today() }));
    console.log(_.map(msgs[3].messages, just_msg));
    t.deepEqual([sends[0].message, sends[1].message, sends[2].message, sends[3].message], _.map(msgs[3].messages, just_msg));
  })
}

// Get a basic date object for right now
function today() {
  var today = new Date();
  var dd: String = String(today.getUTCDate());
  var mm: String = String(today.getUTCMonth() + 1); //January is 0!
  var yyyy: String = String(today.getUTCFullYear());
  return { year: yyyy, month: mm, day: dd }
}