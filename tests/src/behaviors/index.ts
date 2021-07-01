import { Orchestrator, tapeExecutor, compose } from '@holochain/tryorama'
import { defaultConfig, gossipTx, signalTx, phasesTx } from './tx-per-second'  // import config and runner here
import { v4 as uuidv4 } from "uuid";

process.on('unhandledRejection', error => {
    console.error('****************************');
    console.error('got unhandledRejection:', error);
    console.error('****************************');
});

const runName = process.argv[2] || "" + Date.now()  // default exam name is just a timestamp
let config = process.argv[3] ? require(process.argv[3]) : defaultConfig  // use imported config or one passed as a test arg

console.log(`Running behavior test id=${runName} with:\n`, config)

// Below this line should not need changes

config.numConductors = config.nodes * config.conductors

const local = false

const middleware = /*config.endpoints
  ? compose(tapeExecutor(require('tape')), groupPlayersByMachine(config.endpoints, config.conductors))
  :*/ undefined

const orchestrator = new Orchestrator({ middleware })

const trial: string = "phases"

if (trial === "gossip") {
    orchestrator.registerScenario('Measuring messages per-second--gossip', async (s, t) => {
        let txCount = 1
        while (true) {
            t.comment(`trial with ${txCount} tx`)
            // bump the scenario UID for each run of the trial so a different DNA hash will be generated
            s._uid = uuidv4();
            const duration = await gossipTx(s, t, config, txCount, local)
            const txPerSecond = txCount / (duration * 1000)
            t.comment(`took ${duration}ms to receive ${txCount} messages through gossip. TPS: ${txPerSecond}`)
            txCount *= 2
        }
    })
} else if (trial === "signal") {
    const period = 60 * 1000  // timeout
    orchestrator.registerScenario('Measuring messages per-second--signals', async (s, t) => {
        let txCount = 100
        let duration
        let txPerSecondAtMax = 0
        t.comment(`trial with a network of ${config.nodes} nodes, ${config.conductors} conductors per node, and ${config.instances} cells per conductor, but only ${config.activeAgents} active agents (cells)`)
        do {
            t.comment(`trial with ${txCount} tx per ${period}ms`)
            // bump the scenario UID for each run of the trial so a different DNA hash will be generated
            s._uid = uuidv4();
            duration = await signalTx(s, t, config, period, txCount, local)
            if (!duration) {
                t.comment(`failed when attempting ${txCount} messages`)
                break;
            } else {
                t.comment(`succeeded when attempting ${txCount} messages in ${duration}`)
                txCount *= 2
            }
        } while (true)
    })
} else if (trial === "phases") {
    const phases = [
        {
            period: 1000 * 60 * 1,
            messages: 135,
            active: 40,
            senders: 40,
        },
/*        {
            period: 1000 * 60 * 1,
            messages: 133,
            active: 10,
            senders: 1,
        }*/
    ]
    orchestrator.registerScenario('Measuring messages per-second--phases', async (s, t) => {
        t.comment(`trial with a network of ${config.nodes} nodes, ${config.conductors} conductors per node, and ${config.instances} cells per conductor, in the following phases: ${JSON.stringify(phases)}`)
        s._uid = uuidv4();
        await phasesTx(s, t, config, phases, local);
    });
}

orchestrator.run()
