#!/bin/bash

cargo run --bin risc0-build-methods
cargo build --release
cargo build --release --manifest-path contract/Cargo.toml
cargo test --release
trunk build --release web/client/index.html
