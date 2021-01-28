import { Orchestrator, tapeExecutor, compose } from '@holochain/tryorama'
import { defaultConfig, gossipTx, signalTx } from './tx-per-second'  // import config and runner here
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

const local = true

const middleware = /*config.endpoints
  ? compose(tapeExecutor(require('tape')), groupPlayersByMachine(config.endpoints, config.conductors))
  :*/ undefined

const orchestrator = new Orchestrator({ middleware })

const trial: string = "signal"

if (trial === "gossip") {
    orchestrator.registerScenario('Measuring messages per-second--gossip', async (s, t) => {
        let txCount = 50
        while (true) {
            t.comment(`trial with ${txCount} tx`)
            // bump the scenario UUID for each run of the trial so a different DNA hash will be generated
            s._uuid = uuidv4();
            const duration = await gossipTx(s, t, config, txCount, local)
            const txPerSecond = txCount / (duration * 1000)
            t.comment(`took ${duration}ms to receive ${txCount} messages through gossip. TPS: ${txPerSecond}`)
            txCount *= 2
        }
    })
} else if (trial === "signal") {
    const period = 30 * 1000  // timeout
    orchestrator.registerScenario('Measuring messages per-second--signals', async (s, t) => {
        let txCount = 10
        let duration
        let txPerSecondAtMax = 0
        t.comment(`trial with ${config.nodes} nodes, ${config.conductors} conductors per node and ${config.instances} cells per conductor`)
        do {
            t.comment(`trial with ${txCount} tx per ${period}ms`)
            // bump the scenario UUID for each run of the trial so a different DNA hash will be generated
            s._uuid = uuidv4();
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
}

orchestrator.run()
