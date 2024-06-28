use std::io::prelude::*;

use harness_macros::get_program;
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
    let program = get_program!();

    // better testing and more dynamic paths for builds
    assert!(!program.id.is_empty())
}
