import { Player } from '@holochain/tryorama'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { DnaPath, Config, InstallAgentsHapps, InstalledAgentHapps } from '@holochain/tryorama'
import { localConductorConfig, networkedConductorConfig} from '../common'
const path = require('path')

const delay = ms => new Promise(r => setTimeout(r, ms))

type Players = Array<Player>

export const defaultConfig = {
    nodes: 1, // Number of machines
    conductors: 2, // Conductors per machine
    instances: 1, // Instances per conductor
    endpoints: null, // Array of endpoints for Trycp
}

const dnaPath : DnaPath = path.join(__dirname, '../../../elemental-chat.dna.gz')

function prettyCellID(id) {
    return JSON.stringify(id[1].hash)
}

const gossipTrial = async (period, playerAgents, channel, messagesToSend) => {
    const sendingCell = playerAgents[0][0][0].cells[0]
    const receivingCell = playerAgents[1][0][0].cells[0]
    const senderId= "0:0"

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

const setup = async(s, t, config, local) => {
    const conductorConfig = local ? localConductorConfig : networkedConductorConfig;

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
    if (local) {
        await s.shareAllNodes(allPlayers);
    }

    console.log(`Creating channel for test:`)
    const happ = playerAgents[0][0][0] // only one happ per agent
    const channel_uuid = uuidv4();
    const channel = { category: "General", uuid: channel_uuid }
    const createChannelResult = await happ.cells[0].call('chat', 'create_channel', { name: `Test Channel`, channel});
    console.log(createChannelResult);
    return {playerAgents, allPlayers, channel}
}

export const gossipTx = async (s, t, config, period, txCount, local) => {
    const {playerAgents, allPlayers, channel} = await setup(s, t, config, local)
    const actual = await gossipTrial(period, playerAgents, channel, txCount)
    for (const i in allPlayers) {
        const conductor = allPlayers[i]
        conductor.shutdown()
    }
    return actual
}


export const signalTx = async (s, t, config, period, txCount, local) => {
    /*

    const actual = await gossipTrial(period, playerAgents, channel, txCount)
    for (const i in allPlayers) {
        const conductor = allPlayers[i]
        conductor.shutdown()
    }
    return actual*/
}
