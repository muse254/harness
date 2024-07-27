#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-../../target}"
set -e
cd "$(dirname $0)"

echo Target set to $TARGET

# create assets path in current working directory
mkdir -p ~/.config/harness

# first we build the harness code
cargo build --features __harness-build --target wasm32-unknown-unknown --release

# then we optimize the wasm for size & send the output file to a normalized destination with a known name
cargo install wasm-opt
wasm-opt -Oz --strip-debug -o ~/.config/harness/harness_code.wasm $TARGET/wasm32-unknown-unknown/release/hello.wasm

# we can have our second pass to build the final wasm
cargo build --target wasm32-unknown-unknown --release

# we can now generate the did file, piping the output to a file
# should find a way to make candid-extractor work with current set-up as it currently looks into also the compiled wasm which breaks
cargo install candid-extractor
candid-extractor $TARGET/wasm32-unknown-unknown/release/hello.wasm > ./src/hello.did

# using dfx we can now try deploy on out local network
dfx deploy hello --network 'http://localhost:23452/' -y