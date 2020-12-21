import { Orchestrator, Config, InstallAgentsHapps } from '@holochain/tryorama'
import path from 'path'

export const RETRY_DELAY = 1000
export const RETRY_COUNT = 10

// Set up a Conductor configuration using the handy `Conductor.config` helper.
// Read the docs for more on configuration.
export const conductorConfig = Config.gen()

import { TransportConfigType, ProxyAcceptConfig, ProxyConfigType } from '@holochain/tryorama'
export const network = {
  bootstrap_service: "https://bootstrap.holo.host",
  transport_pool: [{
    type: TransportConfigType.Proxy,
    sub_transport: {type: TransportConfigType.Quic},
    proxy_config: {
      type: ProxyConfigType.RemoteProxyClient,
      proxy_url: "kitsune-proxy://CIW6PxKxsPPlcuvUCbMcKwUpaMSmB7kLD8xyyj4mqcw/kitsune-quic/h/proxy.holochain.org/p/5778/--",
    }
  }],
}

export const networkedConductorConfig = Config.gen({network})


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
