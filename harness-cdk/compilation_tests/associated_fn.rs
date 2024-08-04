use harness_cdk::prelude::*;

struct Unit;

impl Unit {
    #[harness]
    fn hello(&self, msg: String) -> String {
        format!("Hello, {msg}!")
    }
}

fn main() {}
