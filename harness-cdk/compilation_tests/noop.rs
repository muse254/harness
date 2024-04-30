use harness_cdk::{harness, harness_export, Encode};

#[harness]
fn noop() {
    println!("Hello stranger!")
}

harness_export!();

fn main() {}
