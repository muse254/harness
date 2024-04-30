#![cfg(feature = "__harness-build")]

use std::assert_eq;

use harness_cdk::{harness, Decode, Encode};
use ic_cdk::query;

#[test]
fn candid_serde() {
    let original_val = (1u8, "One".to_string());
    // encode value
    let val = Encode!(&original_val).unwrap();
    // decode value
    let val = Decode!(&val, (u8, String)).unwrap();
    assert_eq!(original_val, val);
}

#[test]
fn simple_function_test() {
    #[harness(strip = ["something", "else"])]
    #[query]
    fn hello(msg: String) -> String {
        format!("Hello, {msg}!")
    }

    let res = __harness_hello(&Encode!(&String::from("World")).unwrap()).unwrap();
    assert_eq!(
        Decode!(&res, String).unwrap(),
        String::from("Hello, World!")
    );
}

#[test]
fn simple_function_test_no_return() {
    #[harness]
    fn hi(name: String) {
        println!("Hi, {name}");
    }

    let res = __harness_hi(&Encode!(&String::from("stranger")).unwrap()).unwrap();
    assert!(res.is_empty());
}
