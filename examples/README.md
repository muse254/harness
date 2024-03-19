# examples

This should provide a feel of what `harness` offers. We will highlight the hello example; the others are left
as an exercise for the developer to explore!

This code sets up a harness compatible canister.

```rust
use harness_cdk::{harness, harness_export};
use ic_cdk::query;

#[harness]
#[query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

harness_export!();
```

Building the example above with the [script](./hello/build.sh) creates a harness wasm asset. This is what we load into our vanilla canister code
and it is what other devices need to execute our canister code.
