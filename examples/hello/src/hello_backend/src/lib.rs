use harness_cdk::prelude::*;

#[harness]
#[query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

harness_export!();
