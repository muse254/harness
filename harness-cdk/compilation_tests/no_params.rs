use harness_cdk::{harness, harness_export, Encode};

#[harness]
fn hi() -> String {
    String::from("Hello stranger!")
}

harness_export!();

fn main() {}
