use candid::CandidType;
use wapc::{errors, WapcHost};
use wapc_codec::messagepack::{deserialize, serialize};

struct Program<const N: usize, const M: usize> {
    code: [u8; N],
    callable_methods: [String; M],
}

struct Method<I: CandidType, O: CandidType> {
    name: String,
    input: I,
    output: O,
}

#[tokio::main]
async fn main() -> Result<(), errors::Error> {
    let engine = wasmtime_provider::WasmtimeEngineProviderBuilder::new()
        .module_bytes(&buf)
        .build()?;
    let guest = WapcHost::new(
        Box::new(engine),
        Some(Box::new(move |_a, _b, _c, _d, _e| Ok(vec![]))),
    )?;

    let callresult = guest.call("echo", &serialize("hello world").unwrap())?;
    let result: String = deserialize(&callresult).unwrap();
    assert_eq!(result, "hello world");
    Ok(())
}
