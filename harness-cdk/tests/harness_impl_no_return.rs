#![cfg(feature = "__harness-build")]

use std::assert_eq;

use harness_cdk::prelude::*;

#[test]
fn simple_function_test_no_return() {
    fn test() {
        #[harness]
        fn hi(name: String) {
            println!("Hi, {name}");
        }

        #[harness]
        fn hello(name: String) -> String {
            println!("Hello, World!");
            return String::new();
        }

        harness_cdk::harness_export!();

        let res = __harness_hi(&harness_cdk::Encode!(&String::from("stranger")).unwrap()).unwrap();
        assert!(res.is_empty());
    }

    test();
}
