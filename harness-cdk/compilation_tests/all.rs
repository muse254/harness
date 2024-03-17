#[rustversion::stable]
#[test]
fn compilation_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("compilation_tests/method_impl.rs");
    t.pass("compilation_tests/free_standing_fn.rs")
}
