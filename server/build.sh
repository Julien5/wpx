#!/usr/bin/env bash

# 2963  rustup target add x86_64-unknown-linux-musl
# 2965  sudo apt install musl-tools
# 2966  cargo build --release --target x86_64-unknown-linux-musl

# called from make-server-package.sh
function main() {
	cargo build --release --target x86_64-unknown-linux-musl
}

main
