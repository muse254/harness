use harness_cdk::prelude::*;

#[harness]
fn hello(msg: String) -> String {
    format!("Hello, {msg}!")
}

harness_export!();
