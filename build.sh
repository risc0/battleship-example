mkdir -p build
(cd risc0; bazelisk build //examples/rust/battleship/core:init)
(cd risc0; bazelisk build //examples/rust/battleship/core:turn)
(cd contract; cargo build --target wasm32-unknown-unknown --release)

