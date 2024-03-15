# harness

This project provides any valid canister with the ability to arbiter canister computations to any device that can provide compute.

> Work is in progress

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

## Arguments for Viability

The project comes with the guarantees the Internet Computer provides; [infinite scaling](https://internetcomputer.org/how-it-works/scalability/) and security while also offering web2 compatibility.

Why not use the Internet Computer directly?

1. Foster adoption and decentralization. The IC needs relatively powerful hardware to run canisters, in our case this happens but also there are companion devices that can be used to run applications; like phones, tablets, and IoT devices.

2. Like most abstractions, the Harness Network should offer good developer experience. It can also be used outside the context of the IC and Canisters, leveraging the candid protocol
it's also multilingual.

PRs/Issues and Proposals for changes are welcome.
