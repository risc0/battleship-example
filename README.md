A demo of battleship using Risc Zero

* risc0 - The submodule for the main repo
* risc0-verify - The submodule for the rust verifier
* contract - The rust code for the smart contract
* service - The prover service (to be written)
* client - The web UI (cmd-line node code for now)

Instructions:

  // In window 1: Run the web server
  cd risc0
  bazelisk run //examples/rust/battleship/web  

  // In window 2: Build + deploy the contract
  cd contract
  cargo build --target wasm32-unknown-unknown --release
  dev-deploy target/wasm32-unknown-unknown/release/battleship_contract.wasm

  // In window 3: Run JS glue logic
  cd client
  npm install
  // Edit all the constants in main.js for contract + near user
  // Run a game init
  node main.js


