use candid::{Decode, Encode};
use harness_cdk::prelude::*;

#[test]
fn simple_function_test() {
    #[harness]
    fn hello(msg: String) -> String {
        format!("Hello, {msg}!")
    }

    harness_export!();

    let res = __harness_hello(&Encode!(&String::from("World")).unwrap()).unwrap();
    assert_eq!(
        Decode!(&res, String).unwrap(),
        String::from("Hello, World!")
    );
}

#[test]
fn candid_serde() {
    let original_val = (1u8, "One".to_string());
    // encode value
    let val = Encode!(&original_val).unwrap();
    // decode value
    let val = Decode!(&val, (u8, String)).unwrap();
    assert_eq!(original_val, val);
}
