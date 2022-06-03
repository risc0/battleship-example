# Battleship (Rust)
*This README includes instructions to install and play RISC Zero Battleship on your own machine. For more information about RISC Zero Battleship, check out the tutorials at [www.risczero.com](https://www.risczero.com). For questions, find us on [Discord](https://discord.gg/risczero).*

`RISC Zero Battleship` is a 2-player hidden information game implemented in Rust on the NEAR Network.

Players produce proofs of game-state and the result of their actions to enable
two players to play fairly with no intermediaries.

## Components

* Web UI (client)
* Prover web service
* NEAR smart contract

## Requirements

### Yew

Follow the instructions for [Yew Project Setup](https://yew.rs/docs/getting-started/introduction).

### NEAR

Install the `near-cli`:
```
npm install -g near-cli
```

Create a NEAR account: https://wallet.testnet.near.org/create

## Running

Deploy the NEAR smart contract:
```
cargo run --bin risc0-build-methods
cd contract
cargo build --release
near dev-deploy target/wasm32-unknown-unknown/release/battleship_contract.wasm
```

NOTE: If you deploy your own smart contract, you'll need to update the code to point to this new contract.

See: https://github.com/risc0/battleship-example/blob/main/web/client/near.js#L16

Launch the web service:
```
cargo run --bin risc0-build-methods
cargo run --bin battleship-web-server --release
```

Launch the web client:
```
cargo run --bin risc0-build-methods
cd web/client
trunk serve --open
```

## Unit tests

```
cargo run --bin risc0-build-methods
cargo test
```
