#[cfg(test)]
use std::println as info;

#[test]
fn test_schema_info() {
    // should be able to compile retrieve schema info
    let program = harness_macros::get_program!();

    let id: String = program.id.into();
    assert!(!id.is_empty());
    assert!(program.wasm.is_none());
    info!("{:?}", program.schema);
}
