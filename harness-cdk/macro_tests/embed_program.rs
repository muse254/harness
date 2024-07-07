use std::io::prelude::*;
use harness_primitives::{program::Program, HARNESS_PATH};

#[test]
fn test_embed_program() {
    if !std::path::Path::new(HARNESS_PATH)
        .join("harness_code.wasm")
        .exists()
    {
        return; // skip the test
    }

    // should be able to compile the program to memory and populate the program struct
    let program = harness_macros::get_program!();

    // better testing and more dynamic paths for builds
    let id: String = program.id.into();
    assert!(!id.is_empty());
}
