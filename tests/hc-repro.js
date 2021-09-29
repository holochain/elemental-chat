const {
  Orchestrator,
  Config,
  TransportConfigType,
  ProxyConfigType,
  combine,
  localOnly
} = require('@holochain/tryorama')

const path = require('path')

// Note: this is a copy of the network config used in ec dna tests
const network = {
  bootstrap_service: 'https://bootstrap.holo.host',
  transport_pool: [
    {
      type: TransportConfigType.Proxy,
      sub_transport: { type: TransportConfigType.Quic },
      proxy_config: {
        type: ProxyConfigType.RemoteProxyClient,
        proxy_url:
          'kitsune-proxy://CIW6PxKxsPPlcuvUCbMcKwUpaMSmB7kLD8xyyj4mqcw/kitsune-quic/h/165.22.32.11/p/5778/--'
      }
    }
  ],
  tuning_params: {
    gossip_strategy: "sharded-gossip",
    gossip_loop_iteration_delay_ms: "1000",
    gossip_outbound_target_mbps: "0.5",
    gossip_inbound_target_mbps: "0.5",
    gossip_historic_outbound_target_mbps: "0.1",
    gossip_historic_inbound_target_mbps: "0.1",
    gossip_peer_on_success_next_gossip_delay_ms: "60000",
    gossip_peer_on_error_next_gossip_delay_ms: "300000",
    gossip_local_sync_delay_ms: "60000",
    gossip_dynamic_arcs: "false",
    gossip_single_storage_arc_per_space: "false",
    default_rpc_single_timeout_ms: "30000",
    default_rpc_multi_remote_agent_count: "3",
    default_rpc_multi_remote_request_grace_ms: "3000",
    agent_info_expires_after_ms: "1200000",
    tls_in_mem_session_storage: "512",
    proxy_keepalive_ms: "120000",
    proxy_to_expire_ms: "300000",
    concurrent_limit_per_thread: "4096",
    tx2_quic_max_idle_timeout_ms: "30000",
    tx2_pool_max_connection_count: "4096",
    tx2_channel_count_per_connection: "16",
    tx2_implicit_timeout_ms: "30000",
    tx2_initial_connect_retry_delay_ms: "200"
  }
}
const wait = ms => new Promise(resolve => setTimeout(resolve, ms))

const orchestrator = new Orchestrator({ middleware: combine(localOnly) })
const conductorConfig = Config.gen({ network })
const awaitIntegration = async(cell) => {
  while (true) {
      const dump = await cell.stateDump()
      console.log("integration dump was:", dump)
      const idump = dump[0].integration_dump
      if (idump.validation_limbo == 0 && idump.integration_limbo == 0) {
          break
      }
      console.log("waiting 5 seconds for integration")
      await wait(5000)
  }
}

orchestrator.registerScenario('Two Chatters', async scenario => {
  // Tryorama: instantiate player conductor
  const [conductor] = await scenario.players([conductorConfig], false)

  await conductor.startup()

  // Tryorama: install elemental chat on both player conductors
  const bundlePath = path.join(__dirname, '..', 'elemental-chat.happ')

  const aliceChatHapp = await conductor.installBundledHapp({ path: bundlePath }, null, 'first_agent')
  const bobboChatHapp = await conductor.installBundledHapp({ path: bundlePath }, null, 'second_agent')
  const carolChatHapp = await conductor.installBundledHapp({ path: bundlePath }, null, 'third_agent')
  const [aliceChat] = aliceChatHapp.cells
  const [bobboChat] = bobboChatHapp.cells
  const [carolChat] = carolChatHapp.cells
  console.log('alice integration 1')
  await awaitIntegration(aliceChat)
  console.log('bobbo integration 1')
  await awaitIntegration(bobboChat)
  console.log('carol integration 1')
  await awaitIntegration(carolChat)

  await aliceChat.call('chat', 'refresh_chatter', null)
  await bobboChat.call('chat', 'refresh_chatter', null)

  console.log('alice integration 2')
  await awaitIntegration(aliceChat)
  await wait(20_000)
  console.log('alice stats', await aliceChat.call('chat', 'stats', { category: 'General' }))
  console.log('bobbo stats', await bobboChat.call('chat', 'stats', { category: 'General' }))
  console.log('carol stats', await carolChat.call('chat', 'stats', { category: 'General' }))
  console.log('bobbo integration 2')
  await awaitIntegration(bobboChat) // never resolves
  console.log('carol integration 2')
  await awaitIntegration(carolChat) // never resolves
})

orchestrator.run()
