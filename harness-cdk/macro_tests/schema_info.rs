#![cfg(feature = "__harness-build")]
#![allow(clippy::large_stack_frames)]

#[test]
fn test_schema_info() {
    // should be able to compile retrieve schema info
    let bin = harness_macros::get_binary__!();
    assert!(!bin.is_empty());
}
