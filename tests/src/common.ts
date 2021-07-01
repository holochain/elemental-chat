import { Orchestrator, Config, InstallAgentsHapps, InstalledHapp } from '@holochain/tryorama'
import * as msgpack from '@msgpack/msgpack';
import path from 'path'

export const RETRY_DELAY = 1000
export const RETRY_COUNT = 16

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
      // proxy_url: "kitsune-proxy://nFCWLsuRC0X31UMv8cJxioL-lBRFQ74UQAsb8qL4XyM/kitsune-quic/h/192.168.0.203/p/5778/--",
//        proxy_url: "kitsune-proxy://CIW6PxKxsPPlcuvUCbMcKwUpaMSmB7kLD8xyyj4mqcw/kitsune-quic/h/147.75.54.129/p/5778/--",
      //proxy_url: "kitsune-proxy://CIW6PxKxsPPlcuvUCbMcKwUpaMSmB7kLD8xyyj4mqcw/kitsune-quic/h/165.22.32.11/p/5778/--",
      //proxy_url:"kitsune-proxy://nFCWLsuRC0X31UMv8cJxioL-lBRFQ74UQAsb8qL4XyM/kitsune-quic/h/192.168.0.203/p/33679/--",
        //proxy_url: "kitsune-proxy://f3gH2VMkJ4qvZJOXx0ccL_Zo5n-s_CnBjSzAsEHHDCA/kitsune-quic/h/164.90.142.115/p/10000/--"  // p1
   }
  }],
  tuning_params: {
      gossip_loop_iteration_delay_ms: 2000, //number // default 10

        /// Default agent count for remote notify. [Default: 5]
        default_notify_remote_agent_count:  5,

        /// Default timeout for remote notify. [Default: 30s]
        default_notify_timeout_ms: 1000 * 30,

        /// Default timeout for rpc single. [Default: 30s]
        default_rpc_single_timeout_ms: 1000 * 30,

        /// Default agent count for rpc multi. [Default: 2]
        default_rpc_multi_remote_agent_count: 2,

        /// Default timeout for rpc multi. [Default: 30s]
        default_rpc_multi_timeout_ms: 1000 * 30,

        /// Default agent expires after milliseconds. [Default: 20 minutes]
        agent_info_expires_after_ms: 1000 * 60 * 20,

        /// Tls in-memory session storage capacity. [Default: 512]
        tls_in_mem_session_storage: 512,

        /// How often should NAT nodes refresh their proxy contract?
        /// [Default: 2 minutes]
        proxy_keepalive_ms: 1000 * 60 * 2,

        /// How often should proxy nodes prune their ProxyTo list?
        /// Note - to function this should be > proxy_keepalive_ms.
        /// [Default: 5 minutes]
        proxy_to_expire_ms: 1000 * 60 * 5,

        /// Mainly used as the for_each_concurrent limit,
        /// this restricts the number of active polled futures
        /// on a single thread.
        concurrent_limit_per_thread: 4096,

        /// tx2 quic max_idle_timeout
        /// [Default: 30 seconds]
        tx2_quic_max_idle_timeout_ms: 1000 * 30,

        /// tx2 pool max connection count
        /// [Default: 4096]
        tx2_pool_max_connection_count: 4096,

        /// tx2 channel count per connection
        /// [Default: 3]
        tx2_channel_count_per_connection: 16,

        /// tx2 timeout used for passive background operations
        /// like reads / responds.
        /// [Default: 30 seconds]
        tx2_implicit_timeout_ms: 1000 * 30,

        /// tx2 initial connect retry delay
        /// (note, this delay is currenty exponentially backed off--
        /// multiplied by 2x on every loop)
        /// [Default: 200 ms]
      tx2_initial_connect_retry_delay_ms: 200
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

export const installAgents = async (conductor, agentNames, num_players, memProofArray?, holo_agent_override?) => {

  const admin = conductor.adminWs()
  console.log(`registering dna for: ${chatDna}`)
  const  dnaHash = await conductor.registerDna({path: chatDna}, conductor.scenarioUID, {skip_proof: !memProofArray, holo_agent_override})

  const agents: Array<InstalledHapp> = await Promise.all(agentNames.map(
    async (agent, i) => {
      console.log(`generating key for: ${agent}:`)
      const agent_key = await admin.generateAgentPubKey()
      console.log(`${agent} pubkey:`, agent_key.toString('base64'))

      let dna = {
        hash: dnaHash,
        nick: 'elemental-chat',
      }
      if (memProofArray) {
        const agentNum = agentNames.length * num_players + i
        console.log(`Using memproof number ${agentNum}: ${memProofArray[agentNum]}`)
        dna["membrane_proof"] = Buffer.from(memProofArray[agentNum], 'base64')
      }

      const req = {
        installed_app_id: `${agent}_chat`,
        agent_key,
        dnas: [dna]
      }
      console.log(`installing happ for: ${agent}`)
      return await conductor._installHapp(req)
    }
  ))
  return agents
}

export const delay = ms => new Promise(r => setTimeout(r, ms))

export const awaitIntegration = async(cell) => {
    while (true) {
        const dump = await cell.stateDump()
        console.log("integration dump was:", dump)
        const idump = dump[0].integration_dump
        if (idump.validation_limbo == 0 && idump.integration_limbo == 0) {
            break
        }
        console.log("waiting 5 seconds for integration")
        await delay(5000)
    }
}

export const consistency = async(cells) => {
  // 20 Seconds.
  const MAX_TIMEOUT = 1000 * 20;
  var total_published = 0;
  for (const cell of cells) {
        const dump = await cell.stateDump()
        const sdump = dump[0].source_chain_dump
        total_published += sdump.published_ops_count;
  }
  while (true) {
    var total_integrated = 0;
    var total_missing = 0;
    for (const cell of cells) {
      const dump = await cell.stateDump()
      console.log("integration dump was:", dump)
      const idump = dump[0].integration_dump
      if(idump.integrated >= total_published) {
        total_integrated += 1;
      } else {
        total_missing += total_published - idump.integrated;
      }
      console.log("Missing ", total_missing, "ops. Waiting 0.5 seconds for integration")
      await delay(500)
    }
    if(cells.length == total_integrated) {
      return;
    }
  }
}

export const awaitPeers = async(cell, count) => {
    while (true) {
        const dump = await cell.stateDump()
        console.log("peer dump was:", dump)
        const peer_dump = dump[0].peer_dump
        if (peer_dump.peers.length >= count) {
            break
        }
        console.log("waiting 5 seconds for peers to reach", count)
        await delay(5000)
    }
}
