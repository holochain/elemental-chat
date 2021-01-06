import { Orchestrator } from '@holochain/tryorama'

const orchestrator = new Orchestrator()

require('./basic-chatting')(orchestrator)
require('./multi-chunk')(orchestrator)
require('./chat-signals')(orchestrator)
require('./chat-stats')(orchestrator)

orchestrator.run()
