#!/bin/bash

# HARNESS_DIR: the directory where the harness code is located

HARNESS_PROJ="${HARNESS_DIR:-./Cargo.toml}"

cargo build --manifest-path $HARNESS_PROJ --features harness-cdk/__harness-build --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/hello_backend.wasm ./assets/