//! [![v](https://img.shields.io/crates/v/polkahub)](https://github.com/akropolisio/polkahub-cli)
//! ![Web3 sponsored](https://github.com/akropolisio/polkahub-cli/blob/master/assets/web3_badge.png "Project supported by web3 foundation grants program")
//!
//!
//! # !Status: Active WIP!
//! 
//! ## This is a CLI tool to add your chain in polkahub registry and deploy it in minutes.
//!
//! ## Prerequisites
//! 
//! This tool is interesting for Substrate or another Rust-based chain developers, more likely 
//! for the Polkadot system, so we assume you already have [rust installed](https://doc.rust-lang.org/cargo/getting-started/installation.html).
//! 
//! ## Usage
//! 
//! Install the binary with cargo
//! 
//! ```bash
//! cargo install polkahub
//! ```
//! 
//! Then you can create repo for your chain as simple as running:
//! 
//! ```bash
//! 
//! cargo polkahub <token> <project-name>
//! 
//! ```
//! 
//! ## Build from source
//! 
//! If you want to build your own binary from source
//! 
//!
//!
//!
//!

use reqwest;

mod parsing;
use parsing::{Project, POLKAHUB_URL};


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let project = Project::new();
    let hub = &format!("{}", POLKAHUB_URL);
    let response = project.send_create_request(hub).await?;
    response.process();

    Ok(())
}
