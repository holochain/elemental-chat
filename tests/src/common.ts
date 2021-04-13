import { Orchestrator, Config, InstallAgentsHapps, InstalledHapp } from '@holochain/tryorama'
import * as msgpack from '@msgpack/msgpack';
import path from 'path'

export const RETRY_DELAY = 1000
export const RETRY_COUNT = 10

// Set up a Conductor configuration using the handy `Conductor.config` helper.
// Read the docs for more on configuration.
export const localConductorConfig = Config.gen()

import { TransportConfigType, ProxyAcceptConfig, ProxyConfigType, NetworkType } from '@holochain/tryorama'
export const network = {
  bootstrap_service: "https://bootstrap-staging.holo.host",
  network_type: NetworkType.QuicBootstrap,
  transport_pool: [{
    type: TransportConfigType.Proxy,
    sub_transport: { type: TransportConfigType.Quic },
    proxy_config: {
      type: ProxyConfigType.RemoteProxyClient,
      proxy_url: "kitsune-proxy://nFCWLsuRC0X31UMv8cJxioL-lBRFQ74UQAsb8qL4XyM/kitsune-quic/h/192.168.0.203/p/5778/--",
//        proxy_url: "kitsune-proxy://CIW6PxKxsPPlcuvUCbMcKwUpaMSmB7kLD8xyyj4mqcw/kitsune-quic/h/147.75.54.129/p/5778/--",
      //proxy_url: "kitsune-proxy://CIW6PxKxsPPlcuvUCbMcKwUpaMSmB7kLD8xyyj4mqcw/kitsune-quic/h/165.22.32.11/p/5778/--",
      //proxy_url:"kitsune-proxy://nFCWLsuRC0X31UMv8cJxioL-lBRFQ74UQAsb8qL4XyM/kitsune-quic/h/192.168.0.203/p/33679/--",
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
export const chatDna = path.join(__dirname, "../../elemental-chat.dna")

// create an InstallAgentsHapps array with your DNAs to tell tryorama what
// to install into the conductor.
export const installation1agent: InstallAgentsHapps = [
  [[chatDna]],
]
export const installation2agent: InstallAgentsHapps = [
  [[chatDna]],
  [[chatDna]],
]

// this mem_proof is a signature of the `{role, record_locator}` payload signing  a holo public key
// The current holo_hosting pub key is `uhCAkfzycXcycd-OS6HQHvhTgeDVjlkFdE2-XHz-f_AC_5xelQX1N`
const MEM_PROOF = Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICR/PJxdzJx345LodAe+FOB4NWOWQV0Tb5cfP5/8AL/nF6VBfU2pdGltZXN0YW1wks5gUzqazhJyV9WqaGVhZGVyX3NlcQmrcHJldl9oZWFkZXLEJ4QpJEIwak+vC8awMx0vdAe8XSbRRage/CuXmCjRhkkTtWWAUUOp8qplbnRyeV90eXBl3gABo0FwcN4AA6JpZACnem9tZV9pZACqdmlzaWJpbGl0ed4AAaZQdWJsaWPAqmVudHJ5X2hhc2jEJ4QhJAf4ZKktdaQZ6JJj4l+UDRCTwspZSchRPYXtwbdRVvyQBnB8ZqRoYXNoxCeEKSSebKOWLx1D9uHxPBkzVjOgm3gtO6w8VkiiEvigSfgTeFWLVN+pc2lnbmF0dXJlxEC+3INgyz2PfsiwtpBpTZIcx0JYVy9t7rYp2HWnK5x9Vw/uITWUzfIO4uaNl6MQppfkraxHLeNZqamjyEtRWggApWVudHJ53gABp1ByZXNlbnTeAAKqZW50cnlfdHlwZaNBcHClZW50cnnEMoKkcm9sZalkZXZlbG9wZXKucmVjb3JkX2xvY2F0b3Kybmljb2xhc0BsdWNrc3VzLmV1", 'base64')

export const MEM_PROOF_BAD_SIG = Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICR/PJxdzJx345LodAe+FOB4NWOWQV0Tb5cfP5/8AL/nF6VBfU2pdGltZXN0YW1wks5gUzqazhJyV9WqaGVhZGVyX3NlcQmrcHJldl9oZWFkZXLEJ4QpJEIwak+vC8awMx0vdAe8XSbRRage/CuXmCjRhkkTtWWAUUOp8qplbnRyeV90eXBl3gABo0FwcN4AA6JpZACnem9tZV9pZACqdmlzaWJpbGl0ed4AAaZQdWJsaWPAqmVudHJ5X2hhc2jEJ4QhJAf4ZKktdaQZ6JJj4l+UDRCTwspZSchRPYXtwbdRVvyQBnB8ZqRoYXNoxCeEKSSebKOWLx1D9uHxPBkzVjOgm3gtO6w8VkiiEvigSfgTeFWLVN+pc2lnbmF0dXJlxEC+3INgyz2PfsiwtpBpTZIcx0JYVy9t7rYp2HWnK5x9Vw/uITWUzfIO4uaNl6MQppfkraxHLeNZqamjzEtRWggApWVudHJ53gABp1ByZXNlbnTeAAKqZW50cnlfdHlwZaNBcHClZW50cnnEMoKkcm9sZalkZXZlbG9wZXKucmVjb3JkX2xvY2F0b3Kybmljb2xhc0BsdWNrc3VzLmV1", 'base64')

export const installAgents = async (conductor, agentNames, memProof?) => {
  if (!memProof) {
    memProof = MEM_PROOF
  }
  const dnas = [
    {
      hash: await conductor.registerDna({path: chatDna}),
      nick: 'elemental-chat',
      membrane_proof: Array.from(memProof),
    }
  ]
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
