import { Orchestrator } from '@holochain/tryorama'

let orchestrator = new Orchestrator()
require('./basic-chatting')(orchestrator)
orchestrator.run()

orchestrator = new Orchestrator()
require('./transient-nodes')(orchestrator)
orchestrator.run()

/* currently not using multi-chunk
orchestrator = new Orchestrator()
require('./multi-chunk')(orchestrator)
orchestrator.run()
*/

orchestrator = new Orchestrator()
require('./chat-signals')(orchestrator)
orchestrator.run()

orchestrator = new Orchestrator()
require('./chat-stats')(orchestrator)
orchestrator.run()

orchestrator = new Orchestrator()
require('./profile')(orchestrator)
orchestrator.run()

orchestrator = new Orchestrator()
require('./membrane-proof')(orchestrator)
orchestrator.run()

orchestrator = new Orchestrator()
require('./unique-registration-code')(orchestrator)
orchestrator.run()

orchestrator = new Orchestrator()
require('./pagination')(orchestrator)
orchestrator.run()
