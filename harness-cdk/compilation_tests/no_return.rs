use candid::{Decode, Encode};
use harness_cdk::prelude::*;

#[harness]
fn hi(name: String) {
    println!("Hi, {name}");
}

harness_export!();

fn main() {}
