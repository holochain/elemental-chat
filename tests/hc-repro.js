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
  bootstrap_service: 'https://bootstrap-staging.holo.host',
  network_type: 'quic_bootstrap',
  transport_pool: [
    {
      type: TransportConfigType.Proxy,
      sub_transport: { type: TransportConfigType.Quic },
      proxy_config: {
        type: ProxyConfigType.RemoteProxyClient,
        proxy_url:
          'kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--'
      }
    }
  ],

  tuning_params: {
    gossip_strategy: "sharded-gossip",
    default_rpc_multi_remote_agent_count: 1,
    gossip_loop_iteration_delay_ms: 2000, // Default was 10
    agent_info_expires_after_ms: 1000 * 60 * 30, // Default was 20 minutes
    tx2_channel_count_per_connection: 16, // Default was 3
    default_rpc_multi_remote_request_grace_ms: 100,
    gossip_single_storage_arc_per_space: true,

    default_notify_remote_agent_count:  5,
    tls_in_mem_session_storage: 512,
    proxy_keepalive_ms: 1000 * 60 * 2,
    proxy_to_expire_ms: 1000 * 60 * 5,
    concurrent_limit_per_thread: 4096,
    tx2_quic_max_idle_timeout_ms: 1000 * 30,
    tx2_pool_max_connection_count: 4096,
    tx2_implicit_timeout_ms: 1000 * 30,
    tx2_initial_connect_retry_delay_ms: 200,
  }
}
const wait = ms => new Promise(resolve => setTimeout(resolve, ms))
const sum = arr => arr.reduce((a, b) => a + b)

const orchestrator = new Orchestrator({ middleware: combine(localOnly) })
const conductorConfig = Config.gen({ network, db_sync_level: "Off" })
const awaitIntegration = async(authorities, cells) => {
  while (true) {
    const cell_dumps = await Promise.all(cells.map(c => c.stateDump()))
    console.log('cell_dumps')
    cell_dumps.forEach(d => console.log(d))
    const total_published = sum(cell_dumps.map(dump => dump[0].source_chain_dump.published_ops_count))
    console.log('total_published', total_published)

    const authority_dumps = await Promise.all(authorities.map(c => c.stateDump()))
    const integration_dumps = authority_dumps.map(dump => dump[0].integration_dump)
    console.log('integration_dumps', integration_dumps)
    if (integration_dumps.every(integration_dump => integration_dump.integrated === total_published && integration_dump.validation_limbo === 0)) {
        break
    }
    console.log("waiting 5 seconds for integration")
    await wait(5000)
  }
}

orchestrator.registerScenario('2 chatters + 1 chatter -- fixed by changing UID?', async scenario => {
  const [conductor, conductor2] = await scenario.players([conductorConfig, conductorConfig], false)

  await conductor.startup()
  await conductor2.startup()

  const bundlePath = path.join(__dirname, '..', 'elemental-chat.happ')

  const aliceChatHapp = await conductor.installBundledHapp({ path: bundlePath }, null, 'first_agent')
  const bobboChatHapp = await conductor.installBundledHapp({ path: bundlePath }, null, 'second_agent')
  const carolChatHapp = await conductor2.installBundledHapp({ path: bundlePath }, null, 'third_agent')
  const [aliceChat] = aliceChatHapp.cells
  const [bobboChat] = bobboChatHapp.cells
  const [carolChat] = carolChatHapp.cells
  await awaitIntegration([aliceChat, carolChat], [aliceChat, bobboChat, carolChat])

  await aliceChat.call('chat', 'refresh_chatter', null)
  await bobboChat.call('chat', 'refresh_chatter', null)
  await carolChat.call('chat', 'refresh_chatter', null)

  await bobboChat.call('chat', 'create_channel', {
    "name": "ab",
    "entry": {
        "category": "General",
        "uuid": "45b27e1e-01a1-4672-a313-a88a6192f333"
    }
  })
  await awaitIntegration([aliceChat, carolChat], [aliceChat, bobboChat, carolChat])

  console.log('alice list_channels', await aliceChat.call('chat', 'list_channels', { category: 'General' }))
  console.log('alice stats', await aliceChat.call('chat', 'stats', { category: 'General' }))
  console.log('bobbo list_channels', await bobboChat.call('chat', 'list_channels', { category: 'General' }))
  console.log('bobbo stats', await bobboChat.call('chat', 'stats', { category: 'General' }))
  console.log('carol list_channels', await carolChat.call('chat', 'list_channels', { category: 'General' }))
  console.log('carol stats', await carolChat.call('chat', 'stats', { category: 'General' }))

  await carolChat.call('chat', 'create_channel', {
    "name": "abc",
    "entry": {
        "category": "General",
        "uuid": "54b27e1e-01a1-4672-a313-a88a6192f333"
    }
  })

  await awaitIntegration([aliceChat, carolChat], [aliceChat, bobboChat, carolChat])

  console.log('alice list_channels', await aliceChat.call('chat', 'list_channels', { category: 'General' }))
  console.log('bobbo list_channels', await bobboChat.call('chat', 'list_channels', { category: 'General' }))
  console.log('carol list_channels', await carolChat.call('chat', 'list_channels', { category: 'General' }))


})

orchestrator.run()
