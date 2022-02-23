
const { Codec } = require("@holo-host/cryptolib");
import * as path from 'path'
import * as msgpack from '@msgpack/msgpack';
import { InstalledHapp, Player } from '@holochain/tryorama';

const dnaConfiguration = {
  role_id: 'elemental-chat',
}
const dnaPath = path.join(__dirname, "../../elemental-chat.dna")
const jcFactoryDna = path.join(__dirname, "../../dnas/joining-code-factory.dna");
const installedAppId = agentName => `${agentName}_chat`

export type Memproof = {
  signed_header: {
    header: any,
    signature: Buffer,
  },
  entry: any
}

export const installJCHapp = async (conductor: Player): Promise<InstalledHapp> => {
  const admin = conductor.adminWs()
  const holo_agent_override = await admin.generateAgentPubKey()
  let happ = await conductor._installHapp({
    installed_app_id: `holo_agent_override`,
    agent_key: holo_agent_override,
    dnas: [{
      hash: await conductor.registerDna({ path: jcFactoryDna }, conductor.scenarioUID),
      role_id: 'jc',
    }]
  })
  return happ
}

export const installAgents = async (conductor: Player, agentNames: string[], jcHapp?: InstalledHapp, memProofMutator = m => m): Promise<InstalledHapp[]> => {
  let holo_agent_override = undefined
  if (!!jcHapp) {
    holo_agent_override = Codec.AgentId.encode(jcHapp.agent)
  }
  console.log(`registering dna for: ${dnaPath}`)
  const dnaHash = await conductor.registerDna({ path: dnaPath }, conductor.scenarioUID, { skip_proof: !jcHapp, holo_agent_override })
  const admin = conductor.adminWs()
  
  const agents: Array<InstalledHapp> = []
  for (const i in agentNames) {
    const agent = agentNames[i]
    console.log(`generating key for: ${agent}:`)
    const agent_key = await admin.generateAgentPubKey()
    console.log(`${agent} pubkey:`, Codec.AgentId.encode(agent_key))

    let dna = {
      hash: dnaHash,
      ...dnaConfiguration
    }
    if (!!jcHapp) {
      const membrane_proof = await jcHapp.cells[0].call('code-generator', 'make_proof', {
        role: "ROLE",
        record_locator: "RECORD_LOCATOR",
        registered_agent: Codec.AgentId.encode(agent_key)
      });
      const mutated = memProofMutator(membrane_proof)
      dna["membrane_proof"] = Array.from(msgpack.encode(mutated))
    }

    const req = {
      installed_app_id: installedAppId(agent),
      agent_key,
      dnas: [dna]
    }
    console.log(`installing happ for: ${agent}`)
    let installed = await conductor._installHapp(req)
    console.log(`${installedAppId(agent)} installed`)
    agents.push(installed)
  }

  return agents
}
