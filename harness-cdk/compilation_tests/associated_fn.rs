use harness_cdk::harness;

struct Unit;

impl Unit {
    #[harness]
    fn hello(&self, msg: String) -> String {
        format!("Hello, {msg}!")
    }
}

fn main() {}
