#![cfg(feature = "__harness-build")]

use core::panic;
use std::{assert_eq, println};

use candid::{Decode, DecoderConfig, Encode};
use harness_cdk::{harness, harness_export};

#[test]
fn candid_serde() {
    let original_val = (1u8, "One".to_string());
    // encode value
    let val = Encode!(&original_val).unwrap();
    // decode value
    let val = Decode!([DecoderConfig::new()]; &val, (u8, String)).unwrap();
    assert_eq!(original_val, val);
}

#[test]
fn simple_function_test() {
    #[harness]
    fn hello(msg: String) -> String {
        format!("Hello, {msg}!")
    }

    harness_export!();

    let res = __harness_hello(&Encode!(&String::from("World")).unwrap()).unwrap();
    assert_eq!(
        Decode!([DecoderConfig::new()]; &res, String).unwrap(),
        String::from("Hello, World!")
    );
}
