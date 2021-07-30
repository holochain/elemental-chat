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
      proxy_url: "kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--",
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
export const MEM_PROOF1= Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICREcSxdIB5vMom0+wtjVdw148AUiJ4UG3PYBNqeWiTGdILUqTOpdGltZXN0YW1wks5gweIozhCG10+qaGVhZGVyX3Nlcc0E8qtwcmV2X2hlYWRlcsQnhCkk66tTpglJFMXJ2NLbKEw7+2GkGgueWoLm0aYXpJji+9iURW0mqmVudHJ5X3R5cGXeAAGjQXBw3gADomlkAKd6b21lX2lkAKp2aXNpYmlsaXR53gABplB1YmxpY8CqZW50cnlfaGFzaMQnhCEkCsVmECMjpUlPSpCjfbY6KAORN2xvjsbVioJNBX1S1mjE/nSNpGhhc2jEJ4QpJJlxD1gzNcr0ekzERoetgdNF/qhT6lXW88lHtnPfsjF1mVrY6qlzaWduYXR1cmXEQNhPPWrYTk3fn4hcK4OEpRWFmLY6z4htczSI53EFpG+ahBdtOfLBoiHOEo+d1VAWwODs4EXI1GHTV8qYCpz0PQqlZW50cnneAAGnUHJlc2VudN4AAqplbnRyeV90eXBlo0FwcKVlbnRyecQpgqRyb2xlpUFETUlOrnJlY29yZF9sb2NhdG9yrTk2MkBob2xvLmhvc3Q=", 'base64')

export const MEM_PROOF2= Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICREcSxdIB5vMom0+wtjVdw148AUiJ4UG3PYBNqeWiTGdILUqTOpdGltZXN0YW1wks5gweIozhPWgP2qaGVhZGVyX3Nlcc0FAatwcmV2X2hlYWRlcsQnhCkk6JXd1q68xGBDF1k3a9KhjUEJq18J/+7ys/8NaDv1ViANJzQ5qmVudHJ5X3R5cGXeAAGjQXBw3gADomlkAKd6b21lX2lkAKp2aXNpYmlsaXR53gABplB1YmxpY8CqZW50cnlfaGFzaMQnhCEkHra+2RNvkEH9V1IERJdWFmGEcEY/h2m+xTExZn4iQcCtgjKmpGhhc2jEJ4QpJI8VjmeZamyCEbMDyqK4hKyeeR3U0KbNkSIQeTk92a5SkUsYn6lzaWduYXR1cmXEQDGPfFXFwYhB83VcMXmU5PU94nlecC/DZ8HEqzELXTb11ZZOsaSESZrdY/u0Uwvv1Yhu6sY3PNkRQLLv24njbQulZW50cnneAAGnUHJlc2VudN4AAqplbnRyeV90eXBlo0FwcKVlbnRyecQpgqRyb2xlpUFETUlOrnJlY29yZF9sb2NhdG9yrTk3N0Bob2xvLmhvc3Q=", 'base64')

export const MEM_PROOF3=Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICREcSxdIB5vMom0+wtjVdw148AUiJ4UG3PYBNqeWiTGdILUqTOpdGltZXN0YW1wks5gweIozhaACNKqaGVhZGVyX3Nlcc0FDatwcmV2X2hlYWRlcsQnhCkkkdb7W5fk6agMB4gFfNHRg6LLL8k8BhgHNTk4o4udUXLzKTD+qmVudHJ5X3R5cGXeAAGjQXBw3gADomlkAKd6b21lX2lkAKp2aXNpYmlsaXR53gABplB1YmxpY8CqZW50cnlfaGFzaMQnhCEkgAnN1kgce+zK92804lgIKPaG54JxEaFwQIrDJg0DenZEKngJpGhhc2jEJ4QpJA8lCzTh4rGj+qGhX2W7ocWAE2sB9LBo0EPj/TULdplaXGnTJ6lzaWduYXR1cmXEQMsqDRsy5J+9kluk85w8YYwsfgOQTIaRVE1jUfFW7ZBfl6IuaS5FK4jDsugDDO9RNsclEbvS7upsN++hlJHYOwOlZW50cnneAAGnUHJlc2VudN4AAqplbnRyeV90eXBlo0FwcKVlbnRyecQpgqRyb2xlpUFETUlOrnJlY29yZF9sb2NhdG9yrTk4OUBob2xvLmhvc3Q=", 'base64')

export const MEM_PROOF_BAD_SIG = Buffer.from("3gACrXNpZ25lZF9oZWFkZXLeAAKmaGVhZGVy3gACp2NvbnRlbnTeAAekdHlwZaZDcmVhdGWmYXV0aG9yxCeEICR/PJxdzJx345LodAe+FOB4NWOWQV0Tb5cfP5/8AL/nF6VBfU2pdGltZXN0YW1wks5gUzqazhJyV9WqaGVhZGVyX3NlcQmrcHJldl9oZWFkZXLEJ4QpJEIwak+vC8awMx0vdAe8XSbRRage/CuXmCjRhkkTtWWAUUOp8qplbnRyeV90eXBl3gABo0FwcN4AA6JpZACnem9tZV9pZACqdmlzaWJpbGl0ed4AAaZQdWJsaWPAqmVudHJ5X2hhc2jEJ4QhJAf4ZKktdaQZ6JJj4l+UDRCTwspZSchRPYXtwbdRVvyQBnB8ZqRoYXNoxCeEKSSebKOWLx1D9uHxPBkzVjOgm3gtO6w8VkiiEvigSfgTeFWLVN+pc2lnbmF0dXJlxEC+3INgyz2PfsiwtpBpTZIcx0JYVy9t7rYp2HWnK5x9Vw/uITWUzfIO4uaNl6MQppfkraxHLeNZqamjzEtRWggApWVudHJ53gABp1ByZXNlbnTeAAKqZW50cnlfdHlwZaNBcHClZW50cnnEMoKkcm9sZalkZXZlbG9wZXKucmVjb3JkX2xvY2F0b3Kybmljb2xhc0BsdWNrc3VzLmV1", 'base64')

export const MEM_PROOF_READ_ONLY = Buffer.from([0])

export const installAgents = async (conductor, agentNames, player_num?, memProofArray?, holo_agent_override?) => {

  const admin = conductor.adminWs()
  console.log(`registering dna for: ${chatDna}`)
  const  dnaHash = await conductor.registerDna({path: chatDna}, conductor.scenarioUID, {skip_proof: !memProofArray, holo_agent_override})

  const agents: Array<InstalledHapp> = []
  for (const i in agentNames) {
    const agent = agentNames[i]
    console.log(`generating key for: ${agent}:`)
    const agent_key = await admin.generateAgentPubKey()
    console.log(`${agent} pubkey:`, agent_key.toString('base64'))

    let dna = {
      hash: dnaHash,
      nick: 'elemental-chat',
    }
    if (memProofArray) {
      dna["membrane_proof"] = Array.from(memProofArray[i])
    }

    const req = {
      installed_app_id: `${agent}_chat`,
      agent_key,
      dnas: [dna]
    }
    console.log(`installing happ for: ${agent}`)
    agents.push(await conductor._installHapp(req))
  }
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
