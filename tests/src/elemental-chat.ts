import { Config } from '@holochain/tryorama'
import * as _ from 'lodash'

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
    const channel = "hello world";
    const channel_hash = await conductor.call('alice', 'chat', 'create_channel', channel);

    // Alice send a message
    const msg_alice = {
      channel_hash: channel_hash,
      content: "Hello from alice :)",
    };
    const msg_hash = await conductor.call('alice', 'chat', 'create_message', msg_alice);

    // wait a bit for bobbo to receive the published messages,
    await delay(10)

    // Bob list the channel
    const channels = await conductor.call('bobbo', 'chat', 'list_channels', null);

    console.log('channels:', channels)
    t.equal(channels.length, 1)

    const msgs_bobbo = await conductor.call('bobbo', 'chat', 'list_messages', channel_hash);

    console.log('bobboResult> Messages from channel: ', msgs_bobbo);
    // Bob should see one messages
    t.equal(msgs_bobbo.length, 1)

    // and alice sees the same thing as bobbo
    t.deepEqual(msgs_bobbo, [{ message: "Hello from alice :)" }])

    // Bob send a message
    const msg_bobbo = {
      channel_hash,
      content: "Hello from bobbo :)",
    };
    await conductor.call('bobbo', 'chat', 'create_message', msg_bobbo);

    // wait a bit for bobbo to receive the published messages,
    await delay(10)

    const byMessage = x => x.message

    // Alice list messages
    const msgs_alice = _.sortBy(
      await conductor.call('alice', 'chat', 'list_messages', channel_hash),
      byMessage
    )
    msgs_alice.sort(x => x.message);
    console.log('AliceResult> Messages from channel: ', msgs_alice);

    // Alice should see two messages
    t.equal(msgs_alice.length, 2)

    // and alice sees the same thing as bobbo
    t.deepEqual(msgs_alice, _.sortBy([{ message: "Hello from alice :)" }, { message: "Hello from bobbo :)" }], byMessage))
  })
}
