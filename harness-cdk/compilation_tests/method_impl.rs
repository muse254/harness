use harness_cdk::harness;

struct Unit;

#[harness]
impl Unit {
    fn hello(&self, msg: String) -> String {
        format!("Hello, {msg}!")
    }
}

fn main() {}
