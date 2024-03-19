use harness_cdk::{harness, harness_export, Decode, Encode};
use ic_cdk::query;

#[harness]
#[query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

harness_export!();
