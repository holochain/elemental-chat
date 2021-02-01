import { Player, DnaPath, Config, InstallAgentsHapps, InstalledAgentHapps, TransportConfigType, ProxyAcceptConfig, ProxyConfigType, Cell } from '@holochain/tryorama'
import { ScenarioApi } from '@holochain/tryorama/lib/api';
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { network as defaultNetworkConfig } from '../common'
const path = require('path')

const delay = ms => new Promise(r => setTimeout(r, ms))

export const defaultConfig = {
    trycpAddresses: [
        "172.26.136.38:9000", // zippy1
        "172.26.38.158:9000", // zippy2
        "172.26.37.152:9000",
        "172.26.55.252:9000",
        "172.26.223.202:9000", // alastar
        "172.26.160.247:9000",
        "172.26.84.233:9000", // katie
        "172.26.187.15:9000", //bekah
        "172.26.201.167:9000", // lucas
        "172.26.44.116:9000" // peeech
        //"172.26.100.202:9000", // timo1
        //"172.26.156.115:9500" // timo2
    ],
    //trycpAddresses: ["localhost:9000", "192.168.0.16:9000"],
    nodes: 10, // Number of machines
    conductors: 1, // Conductors per machine
    instances: 1, // Instances per conductor
    activeAgents: 5, // Number of agents to consider "active" for chatting
    dnaSource: path.join(__dirname, '../../../elemental-chat.dna.gz'),
    // dnaSource: { url: "https://github.com/holochain/elemental-chat/releases/download/v0.0.1-alpha15/elemental-chat.dna.gz" },
}

type Agents = Array<{ hAppId: string, agent: Buffer, cell: Cell }>
type PlayerAgents = Array<Agents>

const selectActiveAgents = (count: number, playerAgents: PlayerAgents): Agents => {
    if (count > playerAgents.length * playerAgents[0].length) {
        throw new Error(`not enough agents to make ${count} active`)
    }
    let res = new Array(count)
    let i = 0
    let playerIdx = 0
    let agentIdx = 0
    while (i < count) {
        res[i] = playerAgents[playerIdx][agentIdx]
        i += 1
        playerIdx += 1
        if (playerIdx === playerAgents.length) {
            playerIdx = 0
            agentIdx += 1
        }
    }
    return res
}

type StateDump = [
    any, // unused
    string // summary data which we parse
]

type StateDumpRelevant = {
    numPeers: number
    opsIntegrated: number,
    opsValidationLimbo: number,
    opsIntegrationLimbo: number,
    elementsAuthored: number,
    opsPublished: number,
}


// Example state dump
// [
//     /*irrelevant json object omitted*/,
//     "--- Cell State Dump Summary ---\nNumber of other peers in p2p store: 0,\nOps: Limbo (validation: 0 integration: 0) Integrated: 7\nElements authored: 3, Ops published: 7"
// ]


const parseStateDump = (stateDump: string): StateDumpRelevant => {
    const [unused, stateDumpRelevant]: StateDump = JSON.parse(stateDump)

    const regex = /^--- Cell State Dump Summary ---\nNumber of other peers in p2p store: (\d+),\nOps: Limbo \(validation: (\d+) integration: (\d+)\) Integrated: (\d+)\nElements authored: (\d+), Ops published: (\d+)$/

    const groups = regex.exec(stateDumpRelevant)

    if (groups === null) {
        throw new Error("failed to parse state dump")
    }

    return {
        numPeers: Number.parseInt(groups[1], 10),
        opsValidationLimbo: Number.parseInt(groups[2], 10),
        opsIntegrationLimbo: Number.parseInt(groups[3], 10),
        opsIntegrated: Number.parseInt(groups[4], 10),
        elementsAuthored: Number.parseInt(groups[5], 10),
        opsPublished: Number.parseInt(groups[6], 10),
    }
}


const setup = async (s: ScenarioApi, t, config, local): Promise<{ activeAgents: Agents, playerAgents: PlayerAgents, allPlayers: Player[], channel: any }> => {
    let network;
    if (local) {
        network = { transport_pool: [], bootstrap_service: undefined }
    } else {
        network = defaultNetworkConfig
    }

    const conductorConfig = Config.gen({ network })

    t.comment(`Preparing playground: initializing conductors and spawning`)

    const installation: InstallAgentsHapps = _.times(config.instances, () => [[config.dnaSource]]);
    const conductorConfigsArray = _.times(config.conductors, () => conductorConfig);

    let allPlayers: Player[]
    let i = 0;

    // remote in config means use trycp server
    if (config.trycpAddresses.length == 0) {
        allPlayers = await s.players(conductorConfigsArray, false)
        await Promise.all(allPlayers.map(player => player.startup(() => { })));
        i = allPlayers.length
    } else {
        allPlayers = []
        while (allPlayers.length / config.conductors < config.nodes) {
            if (i >= config.trycpAddresses.length) {
                throw new Error(`ran out of trycp addresses after contacting ${allPlayers.length / config.conductors} nodes`)
            }
            let players: Player[];
            try {
                players = await s.players(conductorConfigsArray, false, config.trycpAddresses[i])
                await Promise.all(players.map(player => player.startup(() => { })));
            } catch (e) {
                console.log(`Skipping trycp node ${config.trycpAddresses[i]} due to error: ${JSON.stringify(e)}`)
                i += 1
                continue
            }
            players.forEach(player => allPlayers.push(player));
            i += 1
        }
    }

    // install chat on all the conductors
    const playerAgents: PlayerAgents = await Promise.all(allPlayers.map(async (player, i) => {
        console.log("installing player", i)
        // console.log("installation", installation)
        const agents = await player.installAgentsHapps(installation)
        return agents.map((happs) => {
            const [{ hAppId, agent, cells: [cell] }] = happs;
            console.log(`DNA HASH: ${cell.cellId[0].toString('base64')}`)
            return { hAppId, agent, cell }
        })
    }))

    if (local) {
        let now = Date.now()
        console.log(`Calling share all nodes ${new Date(now).toLocaleString("en-US")}`)
        await s.shareAllNodes(allPlayers);
        console.log(`Finished share all nodes ${new Date(Date.now()).toLocaleString("en-US")}`)
    }

    console.log(`Creating channel for test:`)
    const channel_uuid = uuidv4();
    const channel = { category: "General", uuid: channel_uuid }
    const createChannelResult = await playerAgents[0][0].cell.call('chat', 'create_channel', { name: `Test Channel`, channel });
    console.log(createChannelResult);

    const activeAgents = selectActiveAgents(config.activeAgents, playerAgents)
    let now = Date.now()

    console.log(`Start calling refresh chatter for ${config.activeAgents} agents at ${new Date(now).toLocaleString("en-US")}`)
    await Promise.all(activeAgents.map(
        agent => agent.cell.call('chat', 'refresh_chatter', null)));
    console.log(`End calling refresh chatter at ${new Date(Date.now()).toLocaleString("en-US")}`)

    now = Date.now()
    console.log(`Start find agents at ${new Date(now).toLocaleString("en-US")}`)
    // wait for all active agents to see all other active agents:
    for (const agentIdx in activeAgents) {
        while (true) {
            const stats = await activeAgents[agentIdx].cell.call('chat', 'agent_stats', null);
            console.log(`waiting for all agents are listed as active from perspective of agent #${agentIdx}`, stats)
            if (stats.agents === config.activeAgents) {
                break;
            }
            await delay(2000)
        }
    }
    const endFindAgents = Date.now()
    console.log(`Found agents at ${new Date(endFindAgents).toLocaleString("en-US")}`)
    console.log(`Took: ${(endFindAgents - now) / 1000}s`)

    return { activeAgents, playerAgents, allPlayers, channel: createChannelResult }
}

const send = async (i, cell, channel, signal: "signal" | "noSignal") => {
    const msg = {
        last_seen: { First: null },
        channel: channel.channel,
        message: {
            uuid: uuidv4(),
            content: `message ${i}`,
        },
        chunk: 0,
    }
    console.log(`creating message ${i}`)
    const messageData = await cell.call('chat', 'create_message', msg)
    console.log(`message created ${i}`)

    if (signal === "signal") {
        console.log(`sending signal ${i}`)
        const r = await cell.call('chat', 'signal_chatters', {
            messageData,
            channelData: channel,
        })
        console.log(`signal sent ${i}`)
    }
}

const sendSerially = async (end: number, sendingCell: Cell, channel, messagesToSend: number) => {
    //    const msDelayBetweenMessage = period/messagesToSend
    for (let i = 0; i < messagesToSend; i++) {
        await send(i, sendingCell, channel, "signal")
        if (Date.now() > end) {
            i = i + 1
            console.log(`Couldn't send all messages in period, sent ${i}`)
            return i
        }
        // console.log(`waiting ${msDelayBetweenMessage}ms`)
        // await delay(msDelayBetweenMessage-20)
    }
    return messagesToSend
}

const sendConcurrently = async (agents: Agents, channel, messagesToSend: number, signal: "signal" | "noSignal") => {
    const messagePromises = new Array(messagesToSend)
    for (let i = 0; i < messagesToSend; i++) {
        messagePromises[i] = send(i, agents[i % (agents.length)].cell, channel, signal)
    }
    await Promise.all(messagePromises)
}

const gossipTrial = async (activeAgents: Agents, playerAgents: PlayerAgents, channel, messagesToSend: number): Promise<number> => {
    const receivingCell = playerAgents[0][0].cell
    const start = Date.now()
    await sendConcurrently(activeAgents, channel, messagesToSend, "noSignal")
    console.log(`Getting messages (should be ${messagesToSend})`)
    let received = 0
    while (true) {
        let justReceived = 0;
        try {
            justReceived = (await receivingCell.call('chat', 'list_messages', { channel: channel.channel, active_chatter: false, chunk: { start: 0, end: 1 } })).messages.length
        } catch (e) {
            console.error("error while checking number of messages received", e)
        }

        if (received !== justReceived) {
            received = justReceived
            console.log(`After ${(Date.now() - start)}ms, receiver got ${received} messages`)
            if (received === messagesToSend) {
                return Date.now() - start
            }
        } else {
            await delay(200)
        }
    }
}

const signalTrial = async (period, activeAgents: Agents, allPlayers: Player[], channel, messagesToSend) => {
    let totalAgents = activeAgents.length
    // Track how many signals each agent has received.
    const receipts: Record<string, number> = {}
    let totalReceived = 0;
    const totalExpected = messagesToSend * (totalAgents - 1) // sender doesn't receive signals
    for (const agent of activeAgents) {
        receipts[agent.agent.toString('base64')] = 0
    }
    console.log("Receipts:", receipts)

    let allReceiptsResolve
    const allReceipts = new Promise<number | undefined>((resolve, reject) => allReceiptsResolve = resolve)

    // setup the signal handler for all the players so we can check
    // if all the signals are returned
    for (let i = 0; i < activeAgents.length; i++) {
        const conductor = allPlayers[i]
        conductor.setSignalHandler((signal) => {
            const { data: { cellId: [dnaHash, agentKey], payload: any } } = signal
            const key = agentKey.toString('base64')
            receipts[key] += 1
            totalReceived += 1
            // console.log(`${key} got signal. Total so far: ${totalReceived}`)
            // console.log(`Received Signal for conductor #${i.toString()}, agentKey ${agentKey.toString('hex')}, agent #${idx}:`, signal.data.payload.signal_payload.messageData.message)
            if (totalReceived === totalExpected) {
                allReceiptsResolve(Date.now())
            }
        })
    }

    const start = Date.now()
    console.log(`Start sending messages at ${new Date(start).toLocaleString("en-US")}`)
    const delayPromise = delay(period).then(() => undefined)
    await sendConcurrently(activeAgents, channel, messagesToSend, "signal")
    console.log(`Finished sending messages at ${new Date(Date.now()).toLocaleString("en-US")}`)
    console.log(`Getting messages (should be ${messagesToSend})`)

    const finishTime: number | undefined = await Promise.race([allReceipts, delayPromise])

    if (finishTime === undefined) {
        console.log(`Didn't receive all messages in period (${period / 1000}s)!`)
        console.log(`Total active agents: ${totalAgents}`)
        //        console.log(`Total agents that received all signals: ${finishedCount} (${(finishedCount/totalAgents*100).toFixed(1)}%)`)
        console.log(`Total messages created: ${messagesToSend}`)
        console.log(`Total signals sent: ${totalExpected}`)
        console.log(`Total signals received: ${totalReceived} (${(totalReceived / totalExpected * 100).toFixed(1)}%)`)
        return undefined
    }

    console.log(`All nodes got all signals!`)
    return finishTime - start
}

export const gossipTx = async (s, t, config, txCount, local) => {
    const { activeAgents: activeAgents, playerAgents, allPlayers, channel } = await setup(s, t, config, local)
    const actual = await gossipTrial(activeAgents, playerAgents, channel, txCount)
    await Promise.all(allPlayers.map(player => player.shutdown()))
    return actual
}

export const signalTx = async (s, t, config, period, txCount, local) => {
    // do the standard setup
    const { activeAgents: activeAgents, allPlayers, channel } = await setup(s, t, config, local)

    const actual = await signalTrial(period, activeAgents, allPlayers, channel, txCount)
    await Promise.all(allPlayers.map(player => player.shutdown()))
    return actual
}
