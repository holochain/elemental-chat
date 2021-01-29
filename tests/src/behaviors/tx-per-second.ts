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
    dnaSource: path.join(__dirname, '../../../elemental-chat.dna.gz'),
    // dnaSource: { url: "https://github.com/holochain/elemental-chat/releases/download/v0.0.1-alpha15/elemental-chat.dna.gz" },
}

type PlayerAgents = Array<Array<{ hAppId: string, agent: Buffer, cell: Cell }>>

const setup = async (s: ScenarioApi, t, config, local): Promise<{ playerAgents: PlayerAgents, allPlayers: Player[], channel: any }> => {
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
        console.log(`Calling share all nodes ${new Date(endFindAgents).toLocaleString("en-US")}`)
        await s.shareAllNodes(allPlayers);
        console.log(`Finished share all nodes ${new Date(endFindAgents).toLocaleString("en-US")}`)
    }

    console.log(`Creating channel for test:`)
    const channel_uuid = uuidv4();
    const channel = { category: "General", uuid: channel_uuid }
    const createChannelResult = await playerAgents[0][0].cell.call('chat', 'create_channel', { name: `Test Channel`, channel });
    console.log(createChannelResult);

    let a = 0;
    for (const player of playerAgents) {
        console.log(`calling refresh chatter from player ${a}`)
        a+=1;
        await Promise.all(player.map(agent => {return agent.cell.call('chat', 'refresh_chatter', null)} ));
    }

    const startFindAgents = Date.now()
    console.log(`Start find agents at ${new Date(startFindAgents).toLocaleString("en-US")}`)
    let p = 0;
    // wait for all agents to be active:
    for (const player of playerAgents) {
        let a = 0;
        for (const agent of player) {
            while (true) {
                const stats = await agent.cell.call('chat', 'agent_stats', null);
                console.log(`player ${p} agent ${a}: waiting for all agents to be listed as active`, stats)
                if (stats.agents === config.nodes * config.conductors * config.instances) {
                    break;
                }
                await delay(2000)
            }
            a+=1;
        }
        p+=1;
    }
    const endFindAgents = Date.now()
    console.log(`Found messages at ${new Date(endFindAgents).toLocaleString("en-US")}`)
    console.log(`Took: ${(endFindAgents-startFindAgents)/1000}s`)

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

const sendConcurrently = async (playerAgents: PlayerAgents, channel, messagesToSend: number, signal: "signal" | "noSignal") => {
    const instances = playerAgents[0].length
    const messagePromises = new Array(messagesToSend)
    for (let i = 0; i < messagesToSend; i++) {
        messagePromises[i] = send(i, playerAgents[Math.floor(i / instances) % playerAgents.length][i % instances].cell, channel, signal)
    }
    await Promise.all(messagePromises)
}

const gossipTrial = async (playerAgents: PlayerAgents, channel, messagesToSend: number): Promise<number> => {
    const receivingCell = playerAgents[0][0].cell
    const start = Date.now()
    await sendConcurrently(playerAgents, channel, messagesToSend, "noSignal")
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

const signalTrial = async (period, playerAgents: PlayerAgents, allPlayers: Player[], channel, messagesToSend) => {
    const numInstances = playerAgents[0].length

    let allReceiptsResolve
    const allReceipts = new Promise<number | undefined>((resolve, reject) => allReceiptsResolve = resolve)


    let finishedCount = 0
    let totalAgents = playerAgents.length * numInstances
    // Track how many signals each agent has received.
    // Initialize each slot in `receipts` to equal how many that agent has sent.
    const receipts: number[] = new Array(totalAgents);
    for (let i = 0; i < totalAgents; i++) {
        receipts[i] = Math.ceil(Math.max(messagesToSend - i, 0) / (playerAgents.length * numInstances))
        if (receipts[i] == messagesToSend) {
            finishedCount += 1
        }
    }
    console.log(receipts)
    // setup the signal handler for all the players so we can check
    // if all the signals are returned
    for (let i = 0; i < playerAgents.length; i++) {
        const conductor = allPlayers[i]
        conductor.setSignalHandler((signal) => {
            const { data: { cellId: [dnaHash, agentKey], payload: any } } = signal
            const instanceIdx = playerAgents[i].findIndex(agent => agent.agent.equals(agentKey))
            const idx = i * numInstances + instanceIdx
            // console.log(`Received Signal for conductor #${i.toString()}, agentKey ${agentKey.toString('hex')}, agent #${idx}:`, signal.data.payload.signal_payload.messageData.message)
            receipts[idx] += 1
            if (receipts[idx] === messagesToSend) {
                finishedCount += 1
                console.log(`agent #${idx} got all messages!`)
                if (finishedCount === totalAgents) {
                    allReceiptsResolve(Date.now())
                }
            }
        })
    }

    const start = Date.now()
    console.log(`Start sending messages at ${new Date(start).toLocaleString("en-US")}`)
    const delayPromise = delay(period).then(() => undefined)
    await sendConcurrently(playerAgents, channel, messagesToSend, "signal")
    console.log(`Finished sending messages at ${new Date(Date.now()).toLocaleString("en-US")}`)
    console.log(`Getting messages (should be ${messagesToSend})`)

    const finishTime: number | undefined = await Promise.race([allReceipts, delayPromise])

    if (finishTime === undefined) {
        console.log(`Didn't receive all messages in period (${period/1000}s)!`)
        console.log(`Total agents: ${totalAgents}`)
        console.log(`Total agents that received all signals: ${finishedCount} (${(finishedCount/totalAgents*100).toFixed(1)}%)`)
        console.log(`Total messages created: ${messagesToSend}`)
        console.log(`Total signals sent: ${(messagesToSend * totalAgents) - messagesToSend}`)
        let totalReceived = 0
        for (let i = 0; i < totalAgents; i++) {
            totalReceived += receipts[i]
        }
        totalReceived -= messagesToSend; // account for messages not sent to self
        console.log(`Total signals received: ${totalReceived} (${(totalReceived/(messagesToSend * totalAgents)*100).toFixed(1)}%)`)
        return undefined
    }

    console.log(`All nodes got all signals!`)
    return finishTime - start
}

const signalTrialOld = async (period, playerAgents: PlayerAgents, allPlayers, channel, messagesToSend) => {
    const sendingCell = playerAgents[0][0].cell

    // wait for all agents to be active:
    do {
        await delay(1000)
        const stats = await sendingCell.call('chat', 'stats', { category: "General" });
        if (stats.agents == playerAgents.length) {
            break;
        }
        console.log("waiting for all conductors to be listed as active", stats)
    } while (true) // TODO fix for multi-instance

    let receipts: { [key: string]: number; } = {};
    for (const i in allPlayers) {
        const conductor = allPlayers[i]
        conductor.setSignalHandler((signal) => {
            const me = i
            console.log(`Received Signal for ${me}:`, signal.data.payload.signal_payload.messageData.message)
            if (!receipts[me]) {
                receipts[me] = 1
            } else {
                receipts[me] += 1
            }
        })
    }
    const start = Date.now()
    const sent = await sendSerially(start + period, sendingCell, channel, messagesToSend)
    if (sent != messagesToSend) {
        return sent
    }
    let received = 0
    do {
        received = 0
        let leastReceived = messagesToSend
        for (const [key, count] of Object.entries(receipts)) {
            if (count == messagesToSend) {
                received += 1
            } else {
                if (count < leastReceived) {
                    leastReceived = count
                }
            }
        }
        if (received == Object.keys(receipts).length) {
            console.log(`All nodes got all signals!`)
            return messagesToSend
        }
        if (Date.now() - start > period) {
            console.log(`Didn't receive all messages in period!`)
            return leastReceived
        }
        await delay(1000)
    } while (true)
}

export const gossipTx = async (s, t, config, txCount, local) => {
    const { playerAgents, allPlayers, channel } = await setup(s, t, config, local)
    const actual = await gossipTrial(playerAgents, channel, txCount)
    await Promise.all(allPlayers.map(player => player.shutdown()))
    return actual
}

export const signalTx = async (s, t, config, period, txCount, local) => {
    // do the standard setup
    const { playerAgents, allPlayers, channel } = await setup(s, t, config, local)

    const actual = await signalTrial(period, playerAgents, allPlayers, channel, txCount)
    await Promise.all(allPlayers.map(player => player.shutdown()))
    return actual
}
