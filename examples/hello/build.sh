#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-../../../../target}"
set -e
cd "$(dirname $0)"

# first we build the harness code
cargo build --features harness-cdk/__harness-build --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/hello_backend.wasm ./assets/

# lastly we build the canister code
# cargo build --features harness-cdk/__harness-build --target wasm32-unknown-unknown --release
# cp $TARGET/wasm32-unknown-unknown/release/hello_backend.wasm ./assets/