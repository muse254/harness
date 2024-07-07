use harness_cdk::{self, harness, harness_export};

#[harness]
fn hello(msg: String) -> String {
    format!("Hello, {msg}!")
}

harness_export!();
