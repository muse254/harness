<div align="center">
	<img width="256" src="assets/magneto-bw.svg" alt="Harness logo">

# Harness

</div>

This framework allows for an IC canister to be piggybacked on IoT devices for:

- additional compute off-chain where reasonable or just
- to provide a bridge between the IoT device and the IC.

## How to use

Let's create a sample hello world application.

```rust
// we import the harness cdk prelude
use harness_cdk::prelude::*;

// we store the arbiter in a thread local variable
thread_local! {
    static ARBITER: RefCell<Vec<Arbiter>> = RefCell::new(Vec::new());
}

// we define a service, important to annotate with the #[harness] attribute
//
// note that we can have only #[harness] attributes without using #[query] or #[update]
// for services only needed by the harness network
#[harness]
#[query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// we can define another service, 


harness_export!();



```

## TODO

- [ ] Node implementations
- [ ] CDK implementation
- [x] Macros base implementation
- [ ] Test examples
- [ ] Running harness on chain

## Release notes

Release notes and unreleased changes can be found in the [CHANGELOG](./CHANGELOG.md).

## Structure of the System

```mermaid
    graph TD;
    Client --> 
    ApplicationCanister --> DeviceA;
    ApplicationCanister --> DeviceB;
    ApplicationCanister --> DeviceC;
```
