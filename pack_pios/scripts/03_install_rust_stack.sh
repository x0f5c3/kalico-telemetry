#!/bin/bash
set -e
apt-get update
apt-get install -y build-essential libssl-dev pkg-config python3 python3-venv python3-pip
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
cd /opt/kalico-telemetry
cargo build --release -p service
install -Dm755 target/release/service /usr/local/bin/kalico-telemetry
cd ffi_py
pip3 install maturin
maturin develop --release
