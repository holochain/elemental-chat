const { Codec } = require("@holo-host/cryptolib");

import { localConductorConfig, awaitIntegration } from './common'
import { installJCHapp, installAgents, Memproof } from './installAgents'

module.exports = async (orchestrator) => {
  orchestrator.registerScenario('membrane proof tests', async (s, t) => {
    const [conductor] = await s.players([localConductorConfig])
    const jcHapp1 = await installJCHapp((await s.players([localConductorConfig]))[0])
    const jcHapp2 = await installJCHapp((await s.players([localConductorConfig]))[0])
    let [alice_chat_happ, bobbo_chat_happ] = await installAgents(conductor,  ["alice", "bob"], jcHapp1)
    const [alice_chat] = alice_chat_happ.cells
    const [bobbo_chat] = bobbo_chat_happ.cells
    t.ok(alice_chat)
    t.ok(bobbo_chat)

    // zome call triggers init
    let channel_list = await alice_chat.call('chat', 'list_channels', { category: "General" });
    console.log('channel_list : ', channel_list)
    t.equal(channel_list.channels.length, 0, 'number of channels succeeded')

    await awaitIntegration(alice_chat)

    // TODO: add back in when the proofs carry that agent ID
    // this second one should fail because the membrane proofs are agent specific
/*    try {
      channel_list = await bobbo_chat.call('chat', 'list_channels', { category: "General" });
      t.fail()
    } catch(e) {
      t.deepEqual(e, {
        type: 'error',
        data: {
          type: 'internal_error',
          data: 'The cell tried to run the initialize zomes callback but failed because Fail(ZomeName("chat"), "membrane proof for uhCkknmyjli8dQ_bh8TwZM1YzoJt4LTusPFZIohL4oEn4E3hVi1Tf already used")'
        }
      })
    }*/

    // now try and install carol with a membrane proof from a different joining code authority
    try {
      await installAgents(conductor,  ["carol"], { ...jcHapp2, agent: jcHapp1.agent })
      t.fail()
    } catch(e) {
      t.deepEqual(e, {
        type: 'error',
        data: {
          type: 'internal_error',
          data: `Conductor returned an error while using a ConductorApi: GenesisFailed { errors: [ConductorApiError(WorkflowError(GenesisFailure(\
"Joining code invalid: unexpected author (AgentPubKey(${Codec.AgentId.encode(jcHapp2.agent)}))")))] }`
        }
      })
    }

    // now install david with a membrane proof that has a mismatched signature
    const corruptMemproofSignature = (memproof: Memproof) => {
      const sig = Array.from(memproof.signed_header.signature)
      sig[sig.length - 1] ^= 1
      const signature = Buffer.from(sig)
      return {
        ...memproof,
        signed_header: {
          ...memproof.signed_header,
          signature
        }
      }
    }
    try {
      await installAgents(conductor,  ["david"], jcHapp1, corruptMemproofSignature)
      t.fail()
    } catch(e) {
      t.deepEqual(e, { type: 'error', data: { type: 'internal_error', data: 'Conductor returned an error while using a ConductorApi: GenesisFailed { errors: [ConductorApiError(WorkflowError(GenesisFailure("Joining code invalid: incorrect signature")))] }' } })
    }
  })
}
