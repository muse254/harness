use harness_cdk::harness;

#[harness]
fn hello(msg: String) -> String {
    format!("Hello, {msg}!")
}

fn main() {}
