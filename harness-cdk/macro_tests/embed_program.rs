#[cfg(test)]
use std::println as info;

use harness_primitives::HARNESS_PATH;

#[test]
fn test_embed_program() {
    if !std::path::Path::new(HARNESS_PATH)
        .join("harness_code.wasm")
        .exists()
    {
        return; // skip the test if no harness program has been compiled yet
    }

    // should be able to compile the program to memory and populate the program struct
    let program = harness_macros::get_program!();

    let id: String = program.id.into();
    assert!(!id.is_empty());
    assert!(!program.wasm.is_empty());

    info!("{:?}", program.schema);
}
