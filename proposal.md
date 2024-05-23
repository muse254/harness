# Proposal for “Harness” (A decentralized compute cloud on handheld, IoT and other smart devices)

Harness is a collection of technologies that lets anyone create a program that can utilize decentralized devices as its cloud. In essence, it's an open-source cloud software that functions on decentralized devices. Why would someone want to use it? Three potential applications for the technology include establishing a decentralised personal server, enabling communication with IoT devices, and serving as a resource for crowd computing.

The project uses [candid](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/candid/candid-concepts) to encode and decode data from its network just like smart contracts on the Internet Computer. Allowing it to be language agnostic. Communication to the smart devices on a “harness” network uses the [waPC protocol](https://wapc.io/docs/spec/).

A harness application uses a two-pass approach to build. This will be written out as a CLI application.

1. In the first pass, we build the harness code and compatible functions. The build target is WebAssembly. The output program will be usable with the waPC protocol.
2. During the second pass, we construct the harness cloud code, a valid Internet Computer canister where the harness code from the first pass is embedded. This cloud code assumes the responsibility of enrolling devices onto the network and acts as an arbiter managing web requests directed to the application.

To become part of the harness network, a device must run a harness node that utilizes the waPC protocol. This node handles program loading and request processing from the cloud canister.

## Further Work

- We can offer this technology in more languages being able to compile to WebAssembly.
- 