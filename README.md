# Demeter Daemon

This is the core component of the Demeter platform. It's a daemon that runs on each k8s cluster (as a STS not a k8s Daemon) and has the following responsibilities:

- serve as entry-point for RPC management interactions with end-users
- synchronize state with peer clusters in the same fabric
- synchronize state with global events from the distributed ledger (aka: blockchain)
- synchronize local k8s state with desired global state as defined by the fabric

For more detail of the above, please refer to the [architecture docs](https://github.com/demeter-run/architecture).

## Usage

This component is meant to be installed as part of the cluster provisioning procedure, please refer to the Demeter [up instructions](https://github.com/demeter-run/up).

If you wish to run this component in isolation, the recommended approach is to use the available [OCI image](https://github.com/demeter-run/daemon/pkgs/container/daemon).

## Development

This component is built using Rust. To develop you'll need the Rust toolchain with version 1.70 or higher. You can build and run the daemon using Cargo as show in the following example:

```sh
cargo run
```