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
use harness_cdk::{harness_export, harness};

// we define a service, annotated with the #[harness] attribute
#[harness]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

harness_export!();
```

## TODO

- [ ] Node implementations
- [x] CDK implementation
- [x] Macros base implementation
- [x] Test examples
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
