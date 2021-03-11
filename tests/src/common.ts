import { Orchestrator, Config, InstallAgentsHapps, InstalledHapp } from '@holochain/tryorama'
import * as msgpack from '@msgpack/msgpack';
import path from 'path'

export const RETRY_DELAY = 1000
export const RETRY_COUNT = 10

// Set up a Conductor configuration using the handy `Conductor.config` helper.
// Read the docs for more on configuration.
export const localConductorConfig = Config.gen()

import { TransportConfigType, ProxyAcceptConfig, ProxyConfigType } from '@holochain/tryorama'
export const network = {
  bootstrap_service: "https://bootstrap.holo.host",
  transport_pool: [{
    type: TransportConfigType.Proxy,
    sub_transport: { type: TransportConfigType.Quic },
    proxy_config: {
        type: ProxyConfigType.RemoteProxyClient,
        //        proxy_url: "kitsune-proxy://A7quSj_YTzwP1DF93QmErksPkDDuDSPT8zBGyhf7MPU/kitsune-quic/h/192.168.1.85/p/58451/--",
//        proxy_url: "kitsune-proxy://CIW6PxKxsPPlcuvUCbMcKwUpaMSmB7kLD8xyyj4mqcw/kitsune-quic/h/147.75.54.129/p/5778/--",
        proxy_url: "kitsune-proxy://CIW6PxKxsPPlcuvUCbMcKwUpaMSmB7kLD8xyyj4mqcw/kitsune-quic/h/165.22.32.11/p/5778/--",
        //proxy_url: "kitsune-proxy://f3gH2VMkJ4qvZJOXx0ccL_Zo5n-s_CnBjSzAsEHHDCA/kitsune-quic/h/164.90.142.115/p/10000/--"  // p1
    }
  }],
  tuning_params: {
      gossip_loop_iteration_delay_ms: 200, //number // default 10
      default_notify_remote_agent_count: 5, //number // default 5
      default_notify_timeout_ms: 100, //number // default 1000
      default_rpc_single_timeout_ms: 20000, // number // default 2000
      default_rpc_multi_remote_agent_count: 2, //number // default 2
      default_rpc_multi_timeout_ms: 2000, //number // default 2000
      agent_info_expires_after_ms: 1000 * 60 * 20, //number // default 1000 * 60 * 20 (20 minutes)
      tls_in_mem_session_storage: 512, // default 512
      proxy_keepalive_ms: 1000 * 30, // default 1000 * 60 * 2 (2 minutes)
      proxy_to_expire_ms:  1000 * 60 * 5 // default 1000 * 60 * 5 (5 minutes)
  }
}

export const networkedConductorConfig = Config.gen({ network })


// Construct proper paths for your DNAs
export const chatDna = path.join(__dirname, "../../elemental-chat.dna.gz")

// create an InstallAgentsHapps array with your DNAs to tell tryorama what
// to install into the conductor.
export const installation1agent: InstallAgentsHapps = [
  [[chatDna]],
]
export const installation2agent: InstallAgentsHapps = [
  [[chatDna]],
  [[chatDna]],
]

const dnas = [
  {
    path: chatDna,
    nick: 'elemental-chat',
    membrane_proof: Array.from(msgpack.encode("Testing...")),
  }
]

export const installAgents = async (conductor, agentNames) => {
  const admin = conductor.adminWs();
  const agents: Array<InstalledHapp> = await Promise.all(agentNames.map(
  async agent => {
      const req = {
        installed_app_id: `${agent}_chat`,
        agent_key: await admin.generateAgentPubKey(),
        dnas
      }
      return await conductor._installHapp(req)
    }
  ))
  return agents
}
