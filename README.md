# Filecoin Bridge

A filecoin bridge base substrate.

## Local Development

Follow these steps to prepare a local development environment:

### Build

Once the development environment is set up, build the node template. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

## Run

### Single Node Development Chain

Purge any existing dev chain state:

```bash
./target/release/filecoin-bridge purge-chain --dev
```

Start a dev chain:

```bash
./target/release/filecoin-bridge --dev
```

Or, start a dev chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/filecoin-bridge -lruntime=debug --dev
```

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

## Template Structure

A Substrate project such as this consists of a number of components that are spread across a few
directories.

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command (`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/filecoin-bridge --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/filecoin-bridge purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```
