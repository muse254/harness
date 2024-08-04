use candid::Encode;
use harness_cdk::prelude::*;

#[harness]
fn hi() -> String {
    String::from("Hello stranger!")
}

harness_export!();

fn main() {}
