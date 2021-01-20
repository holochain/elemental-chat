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

const middleware = /*config.endpoints
  ? compose(tapeExecutor(require('tape')), groupPlayersByMachine(config.endpoints, config.conductors))
  :*/ undefined

const orchestrator = new Orchestrator({ middleware })

const doTxTrial = async (s, t, behavior, local) => {
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
        actual = await behavior(s, t, config, period, txCount, local)  // run runner :-)
        const txPerSecond = actual / period * 1000
        if (txPerSecond > txPerSecondAtMax) {
            txAtMax = txCount
            txPerSecondAtMax = txPerSecond
        }
    } while (txCount == actual)

    t.comment(`test context: ${config.numConductors} total conductors`)
    t.comment(`maxed message per second when sending ${txAtMax}: ${txPerSecondAtMax.toFixed(1)} (sent over ${period / 1000}s)`)
    t.comment(`failed when attempting ${txCount} messages`)
}

/*
orchestrator.registerScenario('Measuring messages per-second--gossip', async (s, t) => {
    await doTxTrial(s, t, gossipTx, true)
})
*/

orchestrator.registerScenario('Measuring messages per-second--signals', async (s, t) => {
    await doTxTrial(s, t, signalTx, true)
})

orchestrator.run()
