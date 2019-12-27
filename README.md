# polkahub

[![v](https://img.shields.io/crates/v/polkahub)](https://github.com/akropolisio/polkahub-cli)
![Web3 sponsored](https://github.com/akropolisio/polkahub-cli/blob/master/img/web3_foundation_grants_badge_black.png "Project supported by web3 foundation grants program")


## !Status: Active WIP!

### polkahub cli for easier blockchain deployment.

### Prerequisites

This tool is interesting for Substrate or another Rust-based chain developers, more likely
for the Polkadot system, so we assume you already have [rust installed](https://doc.rust-lang.org/cargo/getting-started/installation.html).

### Install
#### Option 1: install with script
```bash
bash <(curl http://get.polkahub.org/ -L)
```
#### Option 2: just add it to cargo index

```bash
cargo install polkahub
```

### Usage

Depending on how you installed it you go either just **`polkahub`** or **`cargo polkahub`** in the next step
and you can create repo for your chain. To explore all the :

```bash
(cargo) polkahub --help
```
or
```bash
(cargo) polkahub help
```

### Build from source

If you want to build your own binary from source, you are welcome to do so!

```bash

git clone https://github.com/akropolisio/polkahub-cli.git \
    && cd polkahub-cli/         \
    && cargo build --release    \
    && sudo cp target/release/polkahub /usr/bin/polkahub \
    && sudo chmod +x /usr/bin/polkahub

```






License: MIT
