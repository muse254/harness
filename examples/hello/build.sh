#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-./target}"
set -e
cd "$(dirname $0)"

# create assets path in current working directory
mkdir -p ./harness_assets

# first we build the harness code
cargo build --features __harness-build --target wasm32-unknown-unknown --release

# then we optimize the wasm for size
cargo install wasm-opt
wasm-opt -Oz --strip-debug -o ./hello_harness.wasm $TARGET/wasm32-unknown-unknown/release/hello.wasm

# we can have our second pass to build the final wasm
cargo build --target wasm32-unknown-unknown --release

# we can now generate the did file, piping the output to a file
cargo install candid-extractor
candid-extractor $TARGET/wasm32-unknown-unknown/release/hello.wasm > ./hello.did

# using dfx we can now try deploy on out local network
dfx deploy --network 'http://localhost:12345/' -y