import { Player } from '@holochain/tryorama'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { DnaPath, Config, InstallAgentsHapps, InstalledAgentHapps } from '@holochain/tryorama'
const path = require('path')

const delay = ms => new Promise(r => setTimeout(r, ms))

type Players = Array<Player>

export const defaultConfig = {
    nodes: 1, // Number of machines
    conductors: 1, // Conductors per machine
    instances: 2, // Instances per conductor
    endpoints: null, // Array of endpoints for Trycp
}

const dnaPath : DnaPath = path.join(__dirname, '../../../elemental-chat.dna.gz')

function prettyCellID(id) {
    return JSON.stringify(id[1].hash)
}

const conductorConfig = Config.gen()

const trial = async (period, playerAgents, cellChannels, messagesToSend) => {
    const sendingCell = playerAgents[0][0][0].cells[0]
    const receivingCell = playerAgents[0][1][0].cells[0]
    const senderId= "0:0"

    const channel= { category: 'General', uuid: cellChannels[senderId] }

    var msgs: any[] = [];
//    const msDelayBetweenMessage = period/messagesToSend
    const start = Date.now()
    for (let i =0; i < messagesToSend; i++) {
        const msg = {
            last_seen: { First: null },
            channel,
            message: {
                uuid: uuidv4(),
                content: `message ${i}`,
            },
            chunk: 0,
        }
        console.log(`sending message ${i}`)
        msgs[i] = await sendingCell.call('chat', 'create_message', msg)
        if (Date.now() - start > period) {
            i = i+1
            console.log(`Couldn't send all messages in period, sent ${i}`)
            return i
        }
        // console.log(`waiting ${msDelayBetweenMessage}ms`)
        // await delay(msDelayBetweenMessage-20)
    }

    console.log(`Getting messages (should be ${messagesToSend})`)

    const messagesReceived = await receivingCell.call('chat', 'list_messages', { channel, active_chatter: false, chunk: {start:0, end: 1} })

    console.log(`Receiver got ${messagesReceived.messages.length} messages`)

    return messagesReceived.messages.length
}

export const behaviorRunner = async (s, t, config, period, txCount) => {
    t.comment(`Preparing playground: initializing conductors and spawning`)
    //const conductorConfigsArray = await batchOfConfigs(config.isRemote, config.conductors, config.instances)


    const installation : InstallAgentsHapps = _.times(config.instances, ()=>{return [[dnaPath]]});
    const conductorConfigsArray = _.times(config.conductors, ()=>{return conductorConfig});
    const allPlayers = await s.players(conductorConfigsArray)

    let playerAgents : InstalledAgentHapps = [];
    // install chat on all the conductors
    for (const i in allPlayers) {
        console.log("player", i)
        const happs = await allPlayers[i].installAgentsHapps(installation)
        playerAgents.push(happs)
    }
    let cellChannels = {}
    for (const i in playerAgents) {
        console.log(`Creating channels for agents on conductor ${i}:`)
        for (const j in playerAgents[i]) {
            const happ = playerAgents[i][j][0] // only one happ per agent
            const channel_uuid = uuidv4();
            const channel = await happ.cells[0].call('chat', 'create_channel', { name: `${i}:${j}'s Test Channel`, channel: { category: "General", uuid: channel_uuid } });
            console.log(channel);
            cellChannels[`${i}:${j}`]= channel_uuid
        }
    }
    const actual = await trial(period, playerAgents, cellChannels, txCount)
    for (const i in allPlayers) {
        const conductor = allPlayers[i]
        conductor.shutdown()
    }
    return actual
}
/*
module.exports = (orchestrator) => {
  orchestrator.registerScenario('fish', async (s, t) => {
    // spawn the conductor process
    const { conductor } = await s.players({ conductor: config })
    await conductor.spawn()
return
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

    // Alice sends another messag
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
*/
// Get a basic date object for right now
function today() {
  var today = new Date();
  var dd: String = String(today.getUTCDate());
  var mm: String = String(today.getUTCMonth() + 1); //January is 0!
  var yyyy: String = String(today.getUTCFullYear());
  return { year: yyyy, month: mm, day: dd }
}
