<p align="center">
  <img src="https://cryptoindus.xyz/fav.png">
</p>

<div align="center">

[![Substrate version](https://img.shields.io/badge/Substrate-2.0.0-brightgreen?logo=Parity%20Substrate)](https://substrate.dev/)
[![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](LICENSE)

</div>

# cryptoindus

# Introduction

CryptoIndus is a marketplace to collect and trade unique, single-edition digital artworks.

We pay more attention to the runtime upgrade provided by the substrate, which can facilitate the rapid iteration of the 
product itself. More importantly, the community-based governance mechanism and tools based on the substrate make us more
confident to make products that meet user needs and fully link user groups.

Each artwork on CryptoIndus is a digital collectible – a digital object secured by cryptography and tracked on the 
blockchain. Empowering artists with a platform to showcase and sell their work securely supported with a public 
blockchain solution.

# Overview

The next wave of Internet business models will come from the crypto world. The creator can bypass the gatekeeper through
the token model and let the fans directly benefit, instead of attracting the audience through a centralized gatekeeper 
who collects high rents and sets self-service rules. Each new business model helps to provide creators with a new set of
digital services and new sources of income. The encryption token model is applied to CRYPTOINDUS art trading activities,
and encryption technology should benefit more people.

Artists create digital artwork that they can tokenise via CRYPTOINDUS. All artwork files are held on IPFS (a distributed
storage solution) in future, these assets are given unique identifiers which can be tracked for chain-of-custody and 
provenance.

# Building

_notice: this guide just for linux_

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install dependencies

```bash
$ apt install -y cmake pkg-config libssl-dev git gcc build-essential git clang libclang-dev
# install wasm for rust nightly
$ rustup update nightly
$ rustup target add wasm32-unknown-unknown --toolchain nightly
```

export a variable for compile wasm

```bash
export WASM_BUILD_TYPE=release
```

Build all native code:

```bash
cargo build
```

# 4. Run

You can start a development chain with:

```bash
cargo run -- --dev -d=<you path>
```
