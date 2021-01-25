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

const trial: "signal" | "gossip" = "gossip"

if (trial === "gossip") {
    orchestrator.registerScenario('Measuring messages per-second--gossip', async (s, t) => {
        let txCount = 4
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
    orchestrator.registerScenario('Measuring messages per-second--signals', async (s, t) => {
        let txCount = 2
        let actual
        const period = 20 * 1000
        let txPerSecondAtMax = 0
        let txAtMax = 0
        do {
            txCount *= 2
            t.comment(`trial with ${txCount} tx per ${period}ms`)
            // bump the scenario UUID for each run of the trial so a different DNA hash will be generated
            s._uuid = uuidv4();
            actual = await signalTx(s, t, config, period, txCount, local)
            const txPerSecond = actual / period * 1000
            if (txPerSecond > txPerSecondAtMax) {
                txAtMax = txCount
                txPerSecondAtMax = txPerSecond
            }
        } while (txCount === actual)

        t.comment(`test context: ${config.numConductors} total conductors`)
        t.comment(`maxed message per second when sending ${txAtMax}: ${txPerSecondAtMax.toFixed(1)} (sent over ${period / 1000}s)`)
        t.comment(`failed when attempting ${txCount} messages`)
    })
}




orchestrator.run()
