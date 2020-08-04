import { Orchestrator } from '@holochain/tryorama'

const orchestrator = new Orchestrator()

require('./element_chat')(orchestrator)

orchestrator.run()

