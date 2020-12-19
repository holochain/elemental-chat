import { Orchestrator, tapeExecutor, compose } from '@holochain/tryorama'
import { defaultConfig, behaviorRunner } from './tx-per-second'  // import config and runner here

const runName = process.argv[2] || ""+Date.now()  // default exam name is just a timestamp
const config = process.argv[3] ? require(process.argv[3]) : defaultConfig  // use imported config or one passed as a test arg

console.log(`Running behavior test id=${runName} with:\n`, config)

// Below this line should not need changes

config.numConductors = config.nodes * config.conductors
config.isRemote = Boolean(config.endpoints)

const middleware = /*config.endpoints
  ? compose(tapeExecutor(require('tape')), groupPlayersByMachine(config.endpoints, config.conductors))
  :*/ undefined

const orchestrator = new Orchestrator({middleware})

orchestrator.registerScenario('Measuring messages per-second', async (s, t) => {

    var txCount = 1
    var actual
    const period = 10*1000
    do {
        txCount *= 2
        t.comment(`trial with ${txCount} tx per ${period}ms`)
        actual = await behaviorRunner(s, t, config, period, txCount)  // run runner :-)
    } while (txCount == actual)

    t.comment(`message per second: ${(actual/period*1000).toFixed(1)} (sent over ${period/1000}s)`)
})

orchestrator.run()
