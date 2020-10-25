import { Orchestrator, tapeExecutor, groupPlayersByMachine, compose } from '@holochain/tryorama'
import { defaultConfig, behaviorRunner } from './tx-per-minute'  // import config and runner here

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

orchestrator.registerScenario('Measuring messages per-minute before slowdown', async (s, t) => {
    const successRate = await behaviorRunner(s, t, config)  // run runner :-)

    console.log(`successRate: ${successRate}%`)
})

orchestrator.run()
