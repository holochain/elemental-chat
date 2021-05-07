import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { RETRY_DELAY, RETRY_COUNT, localConductorConfig, networkedConductorConfig, installAgents, MEM_PROOF_BAD_SIG, MEM_PROOF1, MEM_PROOF2, awaitIntegration, delay } from './common'

module.exports = async (orchestrator) => {

  orchestrator.registerScenario('membrane proof tests', async (s, t) => {
    const [conductor] = await s.players([localConductorConfig])
    let [alice_chat_happ, bobbo_chat_happ] = await installAgents(conductor,  ["alice", "bob"], [MEM_PROOF1,  MEM_PROOF1])
    const [alice_chat] = alice_chat_happ.cells
    const [bobbo_chat] = bobbo_chat_happ.cells
    // zome call triggers init
    let channel_list = await alice_chat.call('chat', 'list_channels', { category: "General" });
    await awaitIntegration(alice_chat)

    // this second one should fail because it will find the first membrane proof
    try {
      channel_list = await bobbo_chat.call('chat', 'list_channels', { category: "General" });
      t.fail()
    } catch(e) {
      t.deepEqual(e, {
        type: 'error',
        data: {
          type: 'internal_error',
          data: 'The cell tried to run the initialize zomes callback but failed because Fail(ZomeName("chat"), "membrane proof for nicolas@lucksus.eu already used")'
        }
      })
    }

    // now try and install carol with a bad membrane proof
    try {
      let [alice_chat_happ] = await installAgents(conductor,  ["carol"], [MEM_PROOF_BAD_SIG])
      t.fail()
    } catch(e) {
      t.deepEqual(e, { type: 'error', data: { type: 'internal_error', data: 'Conductor returned an error while using a ConductorApi: GenesisFailed { errors: [ConductorApiError(WorkflowError(GenesisFailure("Joining code invalid: incorrect signature")))] }' } })
    }
  })

}
