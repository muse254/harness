use harness_cdk::{harness, harness_export};

#[harness]
fn hi(name: String) {
    println!("Hi, {name}");
}

harness_export!();

fn main() {}
