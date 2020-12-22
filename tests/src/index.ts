import { Orchestrator } from '@holochain/tryorama'

const orchestrator = new Orchestrator()

// require('./elemental-chat')(orchestrator)
// require('./chat-signals')(orchestrator)
require('./chat-stats')(orchestrator)

orchestrator.run()
