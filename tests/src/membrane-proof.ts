import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'
import * as _ from 'lodash'
import { v4 as uuidv4 } from "uuid";
import { RETRY_DELAY, RETRY_COUNT, localConductorConfig, networkedConductorConfig, installAgents, MEM_PROOF_BAD_SIG, MEM_PROOF1, MEM_PROOF2, MEM_PROOF_READ_ONLY, awaitIntegration, delay } from './common'

module.exports = async (orchestrator) => {

  orchestrator.registerScenario('membrane proof tests', async (s, t) => {
    const [conductor] = await s.players([localConductorConfig])
    let [alice_chat_happ, bobbo_chat_happ] = await installAgents(conductor,  ["alice", "bob"], [MEM_PROOF1,  MEM_PROOF1])
    const [alice_chat] = alice_chat_happ.cells
    const [bobbo_chat] = bobbo_chat_happ.cells
    // zome call triggers init
    let channel_list = await alice_chat.call('chat', 'list_channels', { category: "General" });
    await awaitIntegration(alice_chat)

    // this second one should fail because the membrane proofs are agent specific
    // TODO: add back in when the proofs carry that agent ID
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

    // now try and install carol with a bad membrane proof
    try {
      let [carol_chat_happ] = await installAgents(conductor,  ["carol"], [MEM_PROOF_BAD_SIG])
      t.fail()
    } catch(e) {
      t.deepEqual(e, { type: 'error', data: { type: 'internal_error', data: 'Conductor returned an error while using a ConductorApi: GenesisFailed { errors: [ConductorApiError(WorkflowError(GenesisFailure("Joining code invalid: incorrect signature")))] }' } })
    }

    // now try and install doug with the read-only membrane proof
    let [doug_chat_happ] = await installAgents(conductor,  ["doug"], [MEM_PROOF_READ_ONLY])
    const [doug_chat] = doug_chat_happ.cells
    // reading the channel list should work
    channel_list = await doug_chat.call('chat', 'list_channels', { category: "General" });

    // creating a channel should fail
    try {
      const channel = await doug_chat.call('chat', 'create_channel', { name: "Test Channel", entry: { category: "General", uuid: "123" } });
      t.fail()
    } catch(e) {
      t.deepEqual(e, { type: 'error', data: { type: 'ribosome_error', data: 'Wasm error while working with Ribosome: Guest("Read only instance")' } })
    }

    let first_message = {
      last_seen: { First: null },
      channel: {category: "General", uuid: "123"},
      chunk: 0,
      entry: {
        uuid: uuidv4(),
        content: 'x'.repeat(1),
      }
    };
    // sending a message should fail
    try {
      const x = await doug_chat.call('chat', 'create_message', first_message);
      t.fail()
    } catch(e) {
      t.deepEqual(e, { type: 'error', data: { type: 'ribosome_error', data: 'Wasm error while working with Ribosome: Guest("Read only instance")' } })
    }

    // Test holo_agent_override properties
    const MEM_PROP_FOR_NEW_HOLO_AGENT = Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICREcSxdIB5vMom0+wtjVdw148AUiJ4UG3PYBNqeWiTGdILUqTOpdGltZXN0YW1wks5gweIkzivzEHGqaGVhZGVyX3Nlcc0BMKtwcmV2X2hlYWRlcsQnhCkks5/HpSpAL3RXYHfpjhAk8ZXayukBa4/54aur1mBaKL95vbeDqmVudHJ5X3R5cGXeAAGjQXBw3gADomlkAKd6b21lX2lkAKp2aXNpYmlsaXR53gABplB1YmxpY8CqZW50cnlfaGFzaMQnhCEkyy3pfmVBc8BkzVX5+jlnJ3TBYFrrdIdGdEMz0170ZSUTdfg9pGhhc2jEJ4QpJI+UES7dIWlQ0LcaXyirSViVBv7mCZr8GbZKBXZ7GxxR5WFvyKlzaWduYXR1cmXEQLpug6Zw3jDRuqiykCLCHrrD6q0XNxXPYe/Nq/Ec4YXY9Q3ISu9HuCC4qnAhAAOY8fcRNBIfe2WSmYfv1b2ViQalZW50cnneAAGnUHJlc2VudN4AAqplbnRyeV90eXBlo0FwcKVlbnRyecQngqRyb2xlpUFETUlOrnJlY29yZF9sb2NhdG9yqzBAaG9sby5ob3N0", 'base64')
    try {
      let [jack_chat_happ] = await installAgents(conductor,  ["jack"], [MEM_PROP_FOR_NEW_HOLO_AGENT], "uhCAkRHEsXSAebzKJtPsLY1XcNePAFIieFBtz2ATanlokxnSC1Kkz")
      t.ok(true)
    } catch(e) {
      t.fail()
    }
  })

}
