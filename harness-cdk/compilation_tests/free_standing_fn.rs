use candid::{Decode, Encode};
use harness_cdk::prelude::*;

#[harness]
fn hello(msg: String) -> String {
    format!("Hello, {msg}!")
}

harness_export!();

fn main() {}
