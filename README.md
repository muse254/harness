# harness

This project creates a system that allows a project to use decentralized devices as it's cloud. This is possible with the use of an internet computer as the load balancer and proxy.

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
