# cryptoindus

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](./LICENSE)

# Introduction

CryptoIndusis a marketplace to collect and trade unique, single-edition digital artworks.

We pay more attention to the runtime upgrade provided by the substrate, which can facilitate the rapid iteration of the product itself. More importantly, the community-based governance mechanism and tools based on the substrate make us more confident to make products that meet user needs and fully link user groups.
Each artwork on CryptoIndus is a digital collectible – a digital object secured by cryptography and tracked on the blockchain. Empowering artists with a platform to showcase and sell their work securely supported with a public blockchain solution.

# Overview

# Building

## NOTE

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
