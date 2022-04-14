mkdir -p build
(cd risc0; bazelisk build //examples/rust/battleship/web)
(cd contract; cargo build --target wasm32-unknown-unknown --release)

