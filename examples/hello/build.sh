#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-./target}"
set -e
cd "$(dirname $0)"

# first we build the harness code
cargo build --features harness-cdk/__harness-build --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/hello_backend.wasm ./assets/

# then we optimize the wasm for size
cargo install wasm-opt
wasm-opt -Oz --strip-debug -o ./assets/hello_harness.wasm ./assets/hello_backend.wasm
#rm ./assets/hello_backend.wasm

# then we build the canister code
# HARNESS_BUILD="./assets/hello_harness.wasm" cargo build --target wasm32-unknown-unknown --release
# cp $TARGET/wasm32-unknown-unknown/release/hello_backend.wasm ./assets/
