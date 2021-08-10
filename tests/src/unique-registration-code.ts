import { localConductorConfig, installAgents, MEM_PROOF_READ_ONLY } from './common'

module.exports = async (orchestrator) => {

  orchestrator.registerScenario('unique joining codes', async (s, t) => {
    const [conductor] = await s.players([localConductorConfig])
    const MEM_PROOF1 = Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICREcSxdIB5vMom0+wtjVdw148AUiJ4UG3PYBNqeWiTGdILUqTOpdGltZXN0YW1wks5gweIkzivzEHGqaGVhZGVyX3Nlcc0BMKtwcmV2X2hlYWRlcsQnhCkks5/HpSpAL3RXYHfpjhAk8ZXayukBa4/54aur1mBaKL95vbeDqmVudHJ5X3R5cGXeAAGjQXBw3gADomlkAKd6b21lX2lkAKp2aXNpYmlsaXR53gABplB1YmxpY8CqZW50cnlfaGFzaMQnhCEkyy3pfmVBc8BkzVX5+jlnJ3TBYFrrdIdGdEMz0170ZSUTdfg9pGhhc2jEJ4QpJI+UES7dIWlQ0LcaXyirSViVBv7mCZr8GbZKBXZ7GxxR5WFvyKlzaWduYXR1cmXEQLpug6Zw3jDRuqiykCLCHrrD6q0XNxXPYe/Nq/Ec4YXY9Q3ISu9HuCC4qnAhAAOY8fcRNBIfe2WSmYfv1b2ViQalZW50cnneAAGnUHJlc2VudN4AAqplbnRyeV90eXBlo0FwcKVlbnRyecQngqRyb2xlpUFETUlOrnJlY29yZF9sb2NhdG9yqzBAaG9sby5ob3N0", "base64")
    const MEM_PROOF2 = Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICREcSxdIB5vMom0+wtjVdw148AUiJ4UG3PYBNqeWiTGdILUqTOpdGltZXN0YW1wks5gweIkzixIo3KqaGVhZGVyX3Nlcc0BMatwcmV2X2hlYWRlcsQnhCkkj5QRLt0haVDQtxpfKKtJWJUG/uYJmvwZtkoFdnsbHFHlYW/IqmVudHJ5X3R5cGXeAAGjQXBw3gADomlkAKd6b21lX2lkAKp2aXNpYmlsaXR53gABplB1YmxpY8CqZW50cnlfaGFzaMQnhCEkcnWUeAP9pcKJDhZ4o4O90LrmS18D+GEzbW+NDjO8Z0wf3/T9pGhhc2jEJ4QpJEtzArTCIZZC+l/TQktzXOl+xrmogg1nMIB3Ft5NjnxRZhC//KlzaWduYXR1cmXEQEAf7f2MAkMgXiD266vMoLihO0nrUSpUQIsnu8v7nZkec7OnDOQ639H6f0MfrGH3kpNetQ4j6YH1QE7X2RLrLgKlZW50cnneAAGnUHJlc2VudN4AAqplbnRyeV90eXBlo0FwcKVlbnRyecQngqRyb2xlpUFETUlOrnJlY29yZF9sb2NhdG9yqzFAaG9sby5ob3N0", "base64")
    const MEM_PROOF3 = Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICREcSxdIB5vMom0+wtjVdw148AUiJ4UG3PYBNqeWiTGdILUqTOpdGltZXN0YW1wks5gweIkziyMlqqqaGVhZGVyX3Nlcc0BMqtwcmV2X2hlYWRlcsQnhCkkS3MCtMIhlkL6X9NCS3Nc6X7GuaiCDWcwgHcW3k2OfFFmEL/8qmVudHJ5X3R5cGXeAAGjQXBw3gADomlkAKd6b21lX2lkAKp2aXNpYmlsaXR53gABplB1YmxpY8CqZW50cnlfaGFzaMQnhCEkNdwxEvRlAVSYhe62yuA+hcSWSDyIGaAGmZhZhldSb6jxs+WgpGhhc2jEJ4QpJImPRZwMoQXBsQPTbolKfV3n/ULdu7UtEMxZZ+fFAWFO1p6fVKlzaWduYXR1cmXEQF9kWMKc3wf8xt65amaTRf2nozajjzPjDOPKSJsdqQ/Y0npHOXAkJiU9Fp26wfFOKEil3mxagMD5zy4HlwGRnAOlZW50cnneAAGnUHJlc2VudN4AAqplbnRyeV90eXBlo0FwcKVlbnRyecQngqRyb2xlpUFETUlOrnJlY29yZF9sb2NhdG9yqzJAaG9sby5ob3N0", "base64")
    let [bob_chat_happ, carol_chat_happ, daniel_chat_happ] = await installAgents(conductor,  ["bob", "carol", "daniel" ], [ MEM_PROOF1, MEM_PROOF2, MEM_PROOF3], "uhCAkRHEsXSAebzKJtPsLY1XcNePAFIieFBtz2ATanlokxnSC1Kkz")
    
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