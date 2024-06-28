#![cfg(feature = "__harness-build")]

#[rustversion::stable]
#[test]
fn compilation_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("compilation_tests/associated_fn.rs");
    t.pass("compilation_tests/free_standing_fn.rs");
    t.compile_fail("compilation_tests/unsupported_impl.rs");
    t.pass("compilation_tests/no_return.rs");
    t.pass("compilation_tests/no_params.rs");
    t.pass("compilation_tests/noop.rs");
}
