use harness_cdk::{harness, harness_export, Decode, Encode};

#[harness]
fn hi(name: String) {
    println!("Hi, {name}");
}

harness_export!();

fn main() {}
