# elemental-chat

[![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
[![Forum](https://img.shields.io/badge/chat-forum%2eholochain%2enet-blue.svg?style=flat-square)](https://forum.holochain.org)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.org)

[![Twitter Follow](https://img.shields.io/twitter/follow/holochain.svg?style=social&label=Follow)](https://twitter.com/holochain)
License: [![License: CAL 1.0](https://img.shields.io/badge/License-CAL%201.0-blue.svg)](https://github.com/holochain/cryptographic-autonomy-license)

The most basic of all possible chat apps.

## Running the Tests

### Prerequisites

- Build the Holochain tools
  - Clone the repo: `git clone https://github.com/holochain/holochain && cd ./holochain`
  - Ensure correct version of rust tool-chain via nix: `nix-shell`
    - You can also install rust from [https://rustup.rs/](https://rustup.rs/) if you don't want to use nix-shell
  - Install conductor binary: `cargo install --path crates/holochain`
  - Install dna-util binary: `cargo install --path crates/dna_util`
- Build the elemental-chat DNA (assumes you are still in the nix shell for correct rust/cargo versions from step above):
  - Clone this repo: `git clone https://github.com/holochain/elemental-chat && cd ./elemental-chat`
  - Build the wasm: `CARGO_TARGET_DIR=target cargo build --release --target wasm32-unknown-unknown`
  - Assemble the DNA: `dna-util -c elemental-chat.dna.workdir`

## Running

```bash
cd elemental-chat/tests
npm install
npm test
```
> `npm test` will also run the build and assemble commands for you.

## Contribute
Holochain is an open source project.  We welcome all sorts of participation and are actively working on increasing surface area to accept it.  Please see our [contributing guidelines](/CONTRIBUTING.md) for our general practices and protocols on participating in the community, as well as specific expectations around things like code formatting, testing practices, continuous integration, etc.

* Connect with us on our [forum](https://forum.holochain.org)

## License
 [![License: CAL 1.0](https://img.shields.io/badge/License-CAL%201.0-blue.svg)](https://github.com/holochain/cryptographic-autonomy-license)

Copyright (C) 2019 - 2020, Holochain Foundation

This program is free software: you can redistribute it and/or modify it under the terms of the license
provided in the LICENSE file (CAL-1.0).  This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
PURPOSE.
