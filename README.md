# harness

This project provides any valid canister with the ability to arbiter canister computations to any device that can provide compute.

> Work in progress

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
