# polkahub

[![v](https://img.shields.io/crates/v/polkahub)](https://github.com/akropolisio/polkahub-cli)
![Web3 sponsored](https://github.com/akropolisio/polkahub-cli/blob/master/img/web3_foundation_grants_badge_black.png "Project supported by web3 foundation grants program")
## !Status: Active WIP!
### polkahub cli for easier blockchain deployment.

#### Prerequisites
MacOS/Linux: none. </br>
Windows: docker utility installed

### **Windows**
On Windows machine you better use it through pre-compiled docker image like this:
```bash
docker run --rm -ti -v $HOME/.polkahub:/tmp/home -e POLKAHUB_HOME=/tmp/home registry.polkahub.org/polkahub-cli:v2 <action> [ARGS]
```

### **MacOS / Linux**
#### Install
##### Option 1: install with script
```bash
bash <(curl http://get.polkahub.org/ -L)
```
This will install polkahub binary in your `/usr/local/bin`(MacOS) or `/usr/bin`(Linux) directory

##### Option 2: if you are a Rust developer you probably already have cargo installed, so just add it to cargo index

```bash
cargo install polkahub
```

#### Usage
Depending on how you installed it you go either just **`polkahub`** or **`cargo polkahub`** in the next step
and you can create repo for your chain.
To explore all the options run:

```bash
(cargo) polkahub --help
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






License: Apache 2.0
