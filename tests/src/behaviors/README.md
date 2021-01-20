# Scalability tests for Elemental Chat

## Instructions

1. Run [trycp_server](https://github.com/holochain/tryorama/tree/develop/crates/trycp_server) on a bunch of different machines
2. Edit `trycp-addresses.ts` to contain the URLs of the trycp servers you're running
3. Edit `defaultConfig` within `tx-per-second.ts` to contain the number of nodes, etc. that you'd like to benchmark
4. Within `src/` run `npm run test:behavior`, it will try increasing amounts of transactions per second, and once it completes it will output:
  ```
  maxed message per second when sending ____: ____ (sent over ____s)
  ```

**Be cautious of overloading the Holochain test proxy server**