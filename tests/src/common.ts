import { Config } from '@holochain/tryorama'

export const RETRY_DELAY = 5000
export const RETRY_COUNT = 16

// Set up a Conductor configuration using the handy `Conductor.config` helper.
// Read the docs for more on configuration.
export const localConductorConfig = Config.gen()

import { TransportConfigType, ProxyConfigType, NetworkType } from '@holochain/tryorama'
export const NETWORK = {
  bootstrap_service: "https://holodev-bootstrap.holo.host/",
  network_type: NetworkType.QuicBootstrap,
  transport_pool: [{
    type: TransportConfigType.Proxy,
    sub_transport: { type: TransportConfigType.Quic },
    proxy_config: {
      type: ProxyConfigType.RemoteProxyClient,
      proxy_url: "kitsune-proxy://f3gH2VMkJ4qvZJOXx0ccL_Zo5n-s_CnBjSzAsEHHDCA/kitsune-quic/h/137.184.142.208/p/5788/--",
    }
  }],
  tuning_params: {
    // holo-nixpkgs settings
    // gossip_strategy: "sharded-gossip",
    // default_rpc_multi_remote_agent_count: 1,
    // gossip_loop_iteration_delay_ms: 2000, // # Default was 10
    // agent_info_expires_after_ms: 1000 * 60 * 30, // # Default was 20 minutes
    tx2_channel_count_per_connection: 16, // # Default was 3
    // default_rpc_multi_remote_request_grace_ms: 10,
    // gossip_single_storage_arc_per_space: true,
    // Test settings
    gossip_loop_iteration_delay_ms: 200, //number // default 10
    default_notify_remote_agent_count: 5, //number // default 5
    default_notify_timeout_ms: 100, //number // default 1000
    default_rpc_single_timeout_ms: 20000, // number // default 2000
    default_rpc_multi_remote_agent_count: 2, //number // default 2
    default_rpc_multi_timeout_ms: 2000, //number // default 2000
    agent_info_expires_after_ms: 1000 * 60 * 20, //number // default 1000 * 60 * 20 (20 minutes)
    tls_in_mem_session_storage: 512, // default 512
    proxy_keepalive_ms: 1000 * 30, // default 1000 * 60 * 2 (2 minutes)
    proxy_to_expire_ms: 1000 * 60 * 5 // default 1000 * 60 * 5 (5 minutes)

  }
}

export const networkedConductorConfig = Config.gen({ NETWORK })

export const delay = ms => new Promise(r => setTimeout(r, ms))

export const awaitIntegration = async (cell) => {
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

export const consistency = async (cells) => {
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
      if (idump.integrated >= total_published) {
        total_integrated += 1;
      } else {
        total_missing += total_published - idump.integrated;
      }
      console.log("Missing ", total_missing, "ops. Waiting 0.5 seconds for integration")
      await delay(500)
    }
    if (cells.length == total_integrated) {
      return;
    }
  }
}

export const awaitPeers = async (cell, count) => {
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
