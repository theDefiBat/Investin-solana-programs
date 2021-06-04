## First time installation

- Install node
- Install npm
- Install the latest Rust stable from https://rustup.rs/
- Install Solana v1.6.6 or later from
  https://docs.solana.com/cli/install-solana-cli-tools



## Run cluster

```bash
$ solana config set --url localhost

```
Solana maintains three public clusters:
- `devnet` - Development cluster with airdrops enabled
- `testnet` - Tour De Sol test cluster without airdrops enabled
- `mainnet-beta` -  Main cluster


## Create CLI Keypair

If this is your first time using the Solana CLI, you will need to generate a new keypair:

```bash
$ solana-keygen new
```

## Start a local Solana cluster:
```bash
$ solana-test-validator
```
**WARNING: `solana-test-validator` is not currently available for native Windows.  Try using WSL, or switch to Linux or macOS**
Listen to transaction logs:
```bash
$ solana logs
```


### Build the on-chain program

There is both a Rust and C version of the on-chain program, whichever is built
last will be the one used when running the example.

```bash
$ npm run build:program-rust
```


### Deploy the on-chain program

```bash
$ solana program deploy dist/program/helloworld.so
```

### Run the JavaScript client

```bash
$ npm run start
```