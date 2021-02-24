import { Player, DnaPath, PlayerConfig, Config, InstallAgentsHapps, InstalledAgentHapps, TransportConfigType, ProxyAcceptConfig, ProxyConfigType, Cell } from '@holochain/tryorama'
import { ScenarioApi } from '@holochain/tryorama/lib/api';
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { network as defaultNetworkConfig } from '../common'
const path = require('path')

const delay = ms => new Promise(r => setTimeout(r, ms))

export const defaultConfig = {
    trycpAddresses: [
        "172.26.136.38:9000", // zippy1 (58f9o0jx7l73xu7vi13oi0yju06644xm5we2a7i8oqbt918o48)
        "172.26.38.158:9000", // zippy2 (k776n3w1jyovyofz38eex8b8piq89159g985owcbm1annz2hg)
        "172.26.146.6:9000", // zippy (noah's) 1l5nm0ylneapp0z7josuk56fivjly21pcwo0t4o86bhsosapla
//        "172.26.6.201:9000", // alastair (rkbpxayrx3b9mrslvp26oz88rw36wzltxaklm00czl5u5mx1w)
//        "172.26.55.252:9000", // alastair 2 (2dbk737jjs2vyc1z0w72tmc0i7loprr8tbq6f1yevpms4msytn)
//        "172.26.206.158:9000", // mary@holo.host :  (25poc70j8u924ovbzz0tnz1atgrcdg0xjmlo095mck96bbkvtt)
        "172.26.147.238:9000", // mary@marycamacho.com: (38oh2q63ob4w2q1783mir5muup993f2m8gk5kthi0w8ljrc4y4)
        "172.26.208.174:9000", // mc@marycamacho.com: (1k73gwsyo1r8hz8trd4sdbghsjt5gi5b7f3w8anf7xlmndgnt4)
//        "172.26.181.23:9000", // mary.camacho@holo.host:  (5xvizkqpupjpu8ottk7sd9chc24k0otjkkv152756a8ph4p3ct)
//        "172.26.57.175:9000", // rob.lyon+derecha@holo.host (4fx7rhi2i0v4nrvufpgdz31a5374jbvto6hkvo4fvl4f79g5dn)
        "172.26.84.233:9000", // katie
        "172.26.201.167:9000", // lucas (3yk1vqbt914t4cou6lrascjr29h7xa36ucyho72adr3fu0h4f7)
        "172.26.32.181:9000", //bekah
//        "172.26.201.167:9000", // lucas
        "172.26.44.116:9000" // peeech
        //"172.26.100.202:9000", // timo1
        //"172.26.156.115:9500" // timo2
    ],
    //trycpAddresses: ["localhost:9000", "192.168.0.16:9000"],
    proxys: [
        "kitsune-proxy://f3gH2VMkJ4qvZJOXx0ccL_Zo5n-s_CnBjSzAsEHHDCA/kitsune-quic/h/164.90.142.115/p/10000/--",
        "kitsune-proxy://duArtq0LtFEUIDZreC2muXEN3ow_G8zISXKJI3hypCA/kitsune-quic/h/138.197.78.45/p/10000/--",
        "kitsune-proxy://sbUgYILMN7QiHkZZAVjR9Njwlb_Fzb8UE0XsmeGEP48/kitsune-quic/h/161.35.182.155/p/10000/--"
    ],
    proxyCount: 2,
    nodes: 7, // Number of machines
    conductors: 10, // Conductors per machine
    instances: 8, // Instances per conductor
    activeAgents: 5, // Number of agents to consider "active" for chatting
    dnaSource: path.join(__dirname, '../../../elemental-chat.dna.gz'),
    // dnaSource: { url: "https://github.com/holochain/elemental-chat/releases/download/v0.0.1-alpha15/elemental-chat.dna.gz" },
}

type Agents = Array<{ hAppId: string, agent: Buffer, cell: Cell, playerIdx: number }>
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


const parseStateDump = ([unused, stateDumpRelevant]: StateDump): StateDumpRelevant => {
    const regex = /^--- Cell State Dump Summary ---\nNumber of other peers in p2p store: (\d+),\nOps: Limbo \(validation: (\d+) integration: (\d+)\) Integrated: (\d+)\nElements authored: (\d+), Ops published: (\d+)/

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

const waitAllPeers = async (totalPeers: number, playerAgents: PlayerAgents, allPlayers: Player[]) => {
    let now = Date.now()
    console.log(`Start waiting for peer stores at ${new Date(now).toLocaleString("en-US")}`)
    // Wait for all agents to have complete peer stores.
    for (const playerIdx in allPlayers) {
        for (const agentIdx in playerAgents[playerIdx]) {
            const player = allPlayers[playerIdx]
            const agent = playerAgents[playerIdx][agentIdx]
            while (true) {
                const stateDumpRes = await player.adminWs().dumpState({ cell_id: agent.cell.cellId })
                console.log('state dump:', stateDumpRes)
                const stateDump = parseStateDump(stateDumpRes)
                console.log(`waiting for all agents are present in peer store of player #${playerIdx} agent #${agentIdx}`, stateDump)
                if (stateDump.numPeers === totalPeers-1) {
                    break
                }
                await delay(5000)
            }
        }
    }
    const endWaitPeers = Date.now()
    console.log(`Finished waiting for peers at ${new Date(endWaitPeers).toLocaleString("en-US")}`)
    took(`Waiting for peer consistency`, now, endWaitPeers)
}

// wait until the active peers can see all the peers
const waitActivePeers = async (percentNeeded: number, totalPeers: number, activeAgents: Agents, allPlayers: Player[]) : Promise<number> => {
    let startWait = Date.now()
    console.log(`Start waiting for peer stores at ${new Date(startWait).toLocaleString("en-US")}`)
    let conductors = {}
    for (const agent of activeAgents) {
        if (!conductors[agent.playerIdx]) {
            conductors[agent.playerIdx] = { cell_id: agent.cell.cellId }
        }
    }
    for (const [playerIdx, param] of Object.entries(conductors)) {
        while (true) {
            const stateDumpRes = await allPlayers[playerIdx].adminWs().dumpState(param)
            const stateDump = parseStateDump(stateDumpRes)
            console.log('state dump:', stateDump)
            console.log(`waiting for ${percentNeeded}% of peers to be present in peer store of player #${playerIdx}, ${time2text(Date.now() - startWait)} so far`, stateDump)
            if (stateDump.numPeers > (totalPeers-1)*(percentNeeded/100)) {
                break
            }
            await delay(5000)
        }
    }

    const endWaitPeers = Date.now()
    console.log(`Finished waiting for peers at ${new Date(endWaitPeers).toLocaleString("en-US")}`)
    took(`Waiting for active peer consistency`, startWait, endWaitPeers)

    return endWaitPeers - startWait
}

const activateAgents = async (count: number, playerAgents: PlayerAgents): Promise<Agents> => {
    const activeAgents = selectActiveAgents(count, playerAgents);
    _activateAgents(activeAgents, playerAgents);
    _waitAgentsActivated(activeAgents);
    return activeAgents;
}

const _activateAgents = async (activeAgents: Agents, playerAgents: PlayerAgents) => {
    let now = Date.now()

    console.log(`Start calling refresh chatter for ${activeAgents.length} agents at ${new Date(now).toLocaleString("en-US")}`)
    await Promise.all(activeAgents.map(
        agent => agent.cell.call('chat', 'refresh_chatter', null)));
    const endRefresh = Date.now();
    console.log(`End calling refresh chatter at ${new Date(endRefresh).toLocaleString("en-US")}`)
    took(`Activating agents`, now, endRefresh)
}

const _waitAgentsActivated = async (activeAgents: Agents) : Promise<number> => {
    let startWait = Date.now()
    console.log(`Start find agents at ${new Date(startWait).toLocaleString("en-US")}`)
    // wait for all active agents to see all other active agents:
    for (const agentIdx in activeAgents) {
        while (true) {
            const stats = await activeAgents[agentIdx].cell.call('chat', 'agent_stats', null);
            console.log(`waiting for #${agentIdx}'s agent_stats to show ${activeAgents.length} as active (${time2text(Date.now() - startWait)} so far), got:`, stats)
            if (stats.agents === activeAgents.length) {
                break;
            }
            await delay(5000)
        }
    }
    const endFindAgents = Date.now()
    console.log(`End find agents at ${new Date(endFindAgents).toLocaleString("en-US")}`)
    took(`Waiting for active agent consistency`, startWait, endFindAgents)
    return endFindAgents - startWait
}

const maxMinAvg = (counts) =>  {
    let min = Math.min(...counts)
    let max = Math.max(...counts)
    let sum = counts.reduce((a, b) => a + b)
    let avg = sum / counts.length
    return {min, max, avg}
}

const doListMessages = async (msg, channel, activeAgents): Promise<Array<number>> => {
    let i = 0;
    const counts : Array<number> = await Promise.all(
        activeAgents.map(async agent => {
            const r = await agent.cell.call('chat', 'list_messages', { channel: channel.channel, active_chatter: false, chunk: {start:0, end: 1} })
            i+=1;
            console.log(`${i}--called list messages for: `, agent.agent.toString('base64'), r.messages.length)
            return r.messages.length
        }))

    const m = maxMinAvg(counts)
    console.log(`${msg}: Min: ${m.min} Max: ${m.max} Avg ${m.avg.toFixed(1)}`)
    return counts
}

const setup = async (s: ScenarioApi, t, config, local): Promise<{ playerAgents: PlayerAgents, allPlayers: Player[], channel: any }> => {
    let network;
    if (local) {
        network = { transport_pool: [], bootstrap_service: undefined }
    } else {
        network = defaultNetworkConfig
    }

    t.comment(`Preparing playground: initializing conductors and spawning`)

    const installation: InstallAgentsHapps = _.times(config.instances, () => [[config.dnaSource]]);

    let conductorConfigsArray: Array<PlayerConfig>   = []
    for (let i = 0; i < config.conductors; i++) {
        network = _.cloneDeep(network)
        network.transport_pool[0].proxy_config.proxy_url = config.proxys[i%config.proxyCount]
        const conductorConfig = Config.gen({network})
        conductorConfigsArray.push(conductorConfig)
    }
    for (let i = 0; i < config.conductors; i++) {
//        console.log("C",conductorConfigsArray[i]())
    }
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
                console.log("Starting players at trycp server:", config.trycpAddresses[i])
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
            return { hAppId, agent, cell, playerIdx: i }
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

    return { playerAgents, allPlayers, channel: createChannelResult }
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

const took = (msg, start, end) => {
    _took(msg, end - start)
}

const _took = (msg, ms) => {
    const r = time2text(ms);
    console.log(`${msg} took: ${r}`)
}

const time2text = (ms) => {
    let r
    if (ms < 1000) {
        r = `${ms}ms`
    } else if (ms < 60000) {
        r = `${(ms/1000).toFixed(1)}s`
    } else {
        r = `${(ms/(60000)).toFixed(2)}m`
    }
    return r
}

const signalTrial = async (period, activeAgents: Agents, allPlayers: Player[], channel, messagesToSend) => {
    let totalActiveAgents = activeAgents.length
    // Track how many signals each agent has received.
    const receipts: Record<string, number> = {}
    let totalReceived = 0;
    const totalExpected = messagesToSend * (totalActiveAgents - 1) // sender doesn't receive signals
    for (const agent of activeAgents) {
        receipts[agent.agent.toString('base64')] = 0
    }
    console.log("Receipts:", receipts)

    let allReceiptsResolve
    const allReceipts = new Promise<number | undefined>((resolve, reject) => allReceiptsResolve = resolve)

    // setup the signal handler for all the players so we can check
    // if all the signals are returned
    for (let i = 0; i < allPlayers.length; i++) {
        const conductor = allPlayers[i]
        conductor.setSignalHandler((signal) => {
            const { data: { cellId: [dnaHash, agentKey], payload: any } } = signal
            const key = agentKey.toString('base64')
            if (key in receipts) {
                receipts[key] += 1
                totalReceived += 1
                console.log(`${key} got signal. Total so far: ${totalReceived}`)
                // console.log(`Received Signal for conductor #${i.toString()}, agentKey ${agentKey.toString('hex')}, agent #${idx}:`, signal.data.payload.signal_payload.messageData.message)
                if (totalReceived === totalExpected) {
                    allReceiptsResolve(Date.now())
                }
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
        console.log(`Total active agents: ${totalActiveAgents}`)
        //        console.log(`Total agents that received all signals: ${finishedCount} (${(finishedCount/totalActiveAgents*100).toFixed(1)}%)`)
        console.log(`Total messages created: ${messagesToSend}`)
        console.log(`Total signals sent: ${totalExpected}`)
        console.log(`Total signals received: ${totalReceived} (${(totalReceived / totalExpected * 100).toFixed(1)}%)`)
        const numPeersPerActiveAgent = await Promise.all(activeAgents.map(async agent =>
            parseStateDump(await allPlayers[agent.playerIdx].adminWs().dumpState({ cell_id: agent.cell.cellId })).numPeers))
        const min = Math.min(...numPeersPerActiveAgent)
        const max = Math.max(...numPeersPerActiveAgent)
        const sum = numPeersPerActiveAgent.reduce((a, b) => a + b)
        const avg = sum / numPeersPerActiveAgent.length
        console.log(`Peers amongst active agents: Min: ${min} Max: ${max} Avg ${avg}`)
        return undefined
    }

    console.log(`All nodes got all signals!`)
    return finishTime - start
}

const sendOnInterval = async (senders: number, agents: Agents, channel, period: number, sendInterval: number) : Promise<number> => {
    let totalSent = 0
    const start = Date.now()
    do {
        const intervalStart = Date.now()
        let messagePromises = new Array(senders)
        for (let i = 0; i < senders; i++) {
            messagePromises[i] = send(i, agents[i].cell, channel, "signal" )
        }
        await Promise.all(messagePromises)
        const intervalDuration = (Date.now() - intervalStart)
        console.log(`Took: ${intervalDuration}ms to send ${senders} messages`)
        totalSent += senders
        if (intervalDuration < sendInterval) {
            console.log(`Waiting ${(sendInterval - intervalDuration)}ms before sending again`)
            await delay(sendInterval - intervalDuration)
        }
    } while((Date.now() - start) < period)
    return totalSent
}

const PEER_CONSISTENCY_PERCENT = 75

const phaseTrial = async (config, phase, playerAgents: PlayerAgents, allPlayers: Player[], channel) => {
    const period = phase.period
    const messagesInPeriod = phase.messages
    const senders = phase.senders

    const totalPeers = config.nodes * config.conductors * config.instances
    const activeAgents = selectActiveAgents(phase.active, playerAgents)
    const peerConsistencyTook = await waitActivePeers(PEER_CONSISTENCY_PERCENT, totalPeers, activeAgents, allPlayers) // need 75% of peers for go
    await _activateAgents(activeAgents, playerAgents)
    const activationConsistencyTook = await _waitAgentsActivated(activeAgents)

    let totalActiveAgents = activeAgents.length
    // Track how many signals are received in various latencies
    const receipts: { [key: number]: number; }  = {}

    let totalSignalsReceived = 0;

    // setup the signal handler for all the players so we can check
    // if all the signals are returned
    for (let i = 0; i < allPlayers.length; i++) {
        const conductor = allPlayers[i]
        conductor.setSignalHandler((signal) => {
            const { data: { cellId: [dnaHash, agentKey], payload: payload } } = signal
            const now = Date.now()
            const latency = now - payload.signal_payload.messageData.createdAt[0]*1000
            let tranch:number
            if (latency < 30000) {
                tranch = 30000
            } else if (latency < 60000) {
                tranch = 60000
            } else {
                tranch = 1000 * 60 * 3
            }

            if (receipts[tranch] === undefined) {
                receipts[tranch] = 1
            } else {
                receipts[tranch]+=1
            }
            const key = agentKey.toString('base64')
            totalSignalsReceived += 1
            console.log(`${key} got signal with latency ${latency}. Total so far: ${totalSignalsReceived}`)
        })
    }
    const sendInterval = period/(messagesInPeriod/senders)
    const start = Date.now()
    console.log(`Phase begins at ${new Date(start).toLocaleString("en-US")}:`)
    console.log(`   1 message per ${senders} sender ${sendInterval}ms for ${period}ms`)
    const totalMessagesSent = await sendOnInterval(senders, activeAgents, channel, period, sendInterval)
    const phaseEnd = Date.now()
    console.log(`Phase ends at ${new Date(phaseEnd).toLocaleString("en-US")}`)
    took(`Sending ${totalMessagesSent} messages`, start, phaseEnd)

    const totalSignalsExpected = totalMessagesSent * (totalActiveAgents - 1) // sender doesn't receive signals
    let curArrived;
    let threeMinListMessages;
    const msToWaitBeforeCallingItQuits = 30000
    do {
        console.log(`Waiting for ${msToWaitBeforeCallingItQuits/1000}s for signals to finish arriving (expecting ${totalSignalsExpected})`)
        curArrived = totalSignalsReceived
        await delay(msToWaitBeforeCallingItQuits)

        if (Date.now() - phaseEnd > (1000*60*3) && !threeMinListMessages) {
            threeMinListMessages = await doListMessages("3 Minutes later: count of list_message of active peers", channel, activeAgents)
        }

    } while (curArrived != totalSignalsReceived)

    let waitingEnd = Date.now()

    console.log(`Waiting for signals ends at ${new Date(waitingEnd).toLocaleString("en-US")}`)
    took(`Waiting for signals`, phaseEnd, waitingEnd)

    const postSignalWatingListMessages = await doListMessages("Post waiting count of list_message of active peers", channel, activeAgents)
    let finalListMessages
    if (maxMinAvg(postSignalWatingListMessages).min != totalMessagesSent) {
        const msWaitForFinalListMessages = 1000*60*5 // 5 min
        console.log(`Waiting ${time2text(msWaitForFinalListMessages)} for final list message`)
        await delay(msWaitForFinalListMessages)

        finalListMessages = await doListMessages("Final: count of list_message of active peers", channel, activeAgents)
    }

    console.log("----------------------------------------------------------")
    console.log("Results ")
    console.log("----------------------------------------------------------")
    console.log(`Nodes: ${config.nodes}\nConductors/Node: ${config.conductors}\nCells/Conductor: ${config.instances}`)
    console.log(`Proxys: ${config.proxyCount}`)
    console.log(`Total total peers in network: ${totalPeers}`)
    console.log(`Total active peers: ${totalActiveAgents}`)
    _took(`Waiting for ${PEER_CONSISTENCY_PERCENT}% peer consistency`, peerConsistencyTook)
    _took("Waiting for active agent consistency", activationConsistencyTook)
    console.log(`Senders: ${senders}`)
    console.log(`Sending 1 message per ${senders} sender ${(sendInterval/1000).toFixed(1)}s for ${(period/1000).toFixed(1)}s`)
    took(`Sending ${totalMessagesSent} messages`, start, phaseEnd)
    console.log(`Total messages sent: ${totalMessagesSent}`)
    console.log(`Total signals sent: ${totalSignalsExpected}`)

    took(`Waiting for signals after sending ended`, phaseEnd, waitingEnd)
    took(`From start of sending till fished waiting for signals`, start, waitingEnd)


    console.log(`Total signals received: ${totalSignalsReceived} (${(totalSignalsReceived / totalSignalsExpected * 100).toFixed(2)}%)`)

    const latencies  = Object.keys(receipts)
    latencies.sort((a, b) => parseInt(a) - parseInt(b));

    for (let i = 0; i < latencies.length; i++) {
        const l = latencies[i];
        const count = receipts[l];
        const percent = (count/totalSignalsReceived * 100).toFixed(1);
        const tranch = parseInt(l)/1000
        console.log(`Latencies in ${tranch}s: ${count} (${percent})%` )
    }

    if (threeMinListMessages) {
        logCounts("threeMinListMessages", threeMinListMessages, totalMessagesSent)
    }
    logCounts("postSignalWatingListMessages", postSignalWatingListMessages, totalMessagesSent)
    if (finalListMessages) {
        logCounts("finalListMessages", finalListMessages, totalMessagesSent)
    }

    const numPeersPerActiveAgent = await Promise.all(activeAgents.map(async agent =>
                                                                      parseStateDump(await allPlayers[agent.playerIdx].adminWs().dumpState({ cell_id: agent.cell.cellId })).numPeers))

    const m = maxMinAvg(numPeersPerActiveAgent)
    console.log(`Final peers count in peer stores of active peers:\nMin: ${m.min}\nMax: ${m.max}\nAvg: ${m.avg.toFixed(1)}`)

}

const logCounts = (msg, messagesByAgent, totalMessages) => {
    const counts: Record<string, number> = {}
    for (const c of messagesByAgent) {
        let tranch:string
        if (c < totalMessages*.25) {
            tranch = "0-25%"
        } else if (c < totalMessages*.75) {
            tranch = "25-75%"
        } else if (c < totalMessages*.99) {
            tranch = "75-99%"
        } else {
            tranch = " 100%"
        }

        if (counts[tranch] === undefined) {
            counts[tranch] = 1
        } else {
            counts[tranch]+=1
        }

    }
    console.log(msg)
    for (const tranch of [" 100%", "75-99%", "25-75%", "0-25%"]) {
        let count = counts[tranch]
        if (count === undefined) {
            count = 0
        }
        const percent =  (count/messagesByAgent.length*100).toFixed(1)
        console.log(`${tranch}: ${count} (${percent}%)`)
    }
}

export const gossipTx = async (s, t, config, txCount, local) => {
    const { playerAgents, allPlayers, channel } = await setup(s, t, config, local)
    const totalPeers = config.nodes * config.conductors * config.instances
    await waitAllPeers(totalPeers, playerAgents, allPlayers)
    const activeAgents = await activateAgents(config.activeAgents, playerAgents)
    const actual = await gossipTrial(activeAgents, playerAgents, channel, txCount)
    await Promise.all(allPlayers.map(player => player.shutdown()))
    return actual
}

export const signalTx = async (s, t, config, period, txCount, local) => {
    // do the standard setup
    const { playerAgents, allPlayers, channel } = await setup(s, t, config, local)
    const totalPeers = config.nodes * config.conductors * config.instances
    await waitAllPeers(totalPeers, playerAgents, allPlayers)
    const activeAgents = await activateAgents(config.activeAgents, playerAgents)

    const actual = await signalTrial(period, activeAgents, allPlayers, channel, txCount)
    await Promise.all(allPlayers.map(player => player.shutdown()))
    return actual
}

export const phasesTx = async (s, t, config, phases, local) => {
    // do the standard setup
    const { playerAgents, allPlayers, channel } = await setup(s, t, config, local)
    for (const phase of phases) {
        await phaseTrial(config, phase, playerAgents, allPlayers, channel)
    }
    await Promise.all(allPlayers.map(player => player.shutdown()))
}
