import { localConductorConfig } from './common'
import { installJCHapp, installAgents } from './installAgents'

module.exports = async (orchestrator) => {

  orchestrator.registerScenario('unique joining codes', async (s, t) => {
    const [conductor] = await s.players([localConductorConfig])
    const jcHapp = await installJCHapp((await s.players([localConductorConfig]))[0])
    let [bob_chat_happ, carol_chat_happ, daniel_chat_happ] = await installAgents(conductor,  ["bob", "carol", "daniel" ], jcHapp)

    const [bob_chat] = bob_chat_happ.cells
    const [carol_chat] = carol_chat_happ.cells
    const [daniel_chat] = daniel_chat_happ.cells

    // try zome call with one read-only instance and 3 separate agents
    let agent_stats
    try {

    // read/write instances:
    agent_stats = await bob_chat.call('chat', 'agent_stats', null);
    console.log('agent_stats after BOB : ', agent_stats);

    agent_stats = await carol_chat.call('chat', 'agent_stats', null);
    console.log('agent_stats after CAROL : ', agent_stats);

    agent_stats = await daniel_chat.call('chat', 'agent_stats', null);
    console.log('agent_stats after DANIEL : ', agent_stats);
    } catch (error) {
        console.error(error)
        t.fail()
    }
    t.ok(agent_stats)
  })
}
