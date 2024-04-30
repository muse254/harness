use harness_cdk::{harness, harness_export, Decode, Encode};

#[harness]
fn hello(msg: String) -> String {
    format!("Hello, {msg}!")
}

harness_export!();

fn main() {}
