# mamorurs-cli
mamorurs-cli is a command-line interface (CLI) tool for publishing agents to the Mamoru chain. It is built with Rust and uses Cargo as its package manager.  


## Prerequisites
- Rust 1.76.0 or later [Rust](https://www.rust-lang.org/tools/install)
- Cargo

## Installation

To install mamorurs-cli, run the following command:

```bash
git clone https://github.com/Mamoru-Foundation/mamorurs-cli.git
cd mamorurs-cli
```

Build the project:

```bash 
make build-rust-release

or 

make install

or

cargo install --git https://github.com/Mamoru-Foundation/mamorurs-cli
```

The binary will be located in the bin directory.

## Usage

Usage
To publish an agent to the Mamoru chain, use the publish command followed by the path to the agent file:

```bash 
mamorurs-cli agent new -n [<name>]
mamorurs-cli login 
mamorurs-cli agent publish --key "<KEY>" --chain-name <CHAIN_NAME>  /path/to/agent_dir/
mamorurs-cli agent launch --key "<KEY>" --chain-name <CHAIN_NAME> --metadata-id <METADATA_ID> /path/to/agent_dir/
mamorurs-cli agent unregister --agent-id <AGENT_ID>
mamorurs-cli agent assign --agent-id <AGENT_ID> --organization-id <ORGANIZATION_ID>
``` 

## Agent build 
Before building an agent, you must install:

```bash
cargo install cargo-component@=0.11.0 --locked
rustup target add wasm32-wasi
```


To build an agent, run the following command:
```bash
 cargo-component build --release
```
**--release** Release mode is required for the agent.

## Configuration

To configure the CLI, you can edit the configuration file located at ~/.mamorurc/settings.toml The configuration file contains the following fields:

- MAMORU_CLI_AUTH0_DOMAIN 
- MAMORU_CLI_AUTH0_CLIENT_ID 
- MAMORU_CLI_AUTH0_AUDIENCE 
- MAMORU_RPC_URL 
- MAMORU_PRIVATE_KEY 
- MAMORU_GAS_LIMIT 
- MAMORU_GRAPHQL_URL 
- MAMORU_CHAIN_ID
- MAMORU_ORGANIZATION_ID


Copy and edit file devnet.settings.toml, this file contains the default values for devnet.:

```bash
cp example.settings.toml ~/.mamorurc/settings.toml
```

## Testing
To run the tests:

```bash
make test
```

## Linting
To lint the code:

```bash
make lint
```

# License
This project is licensed under the MIT License.
